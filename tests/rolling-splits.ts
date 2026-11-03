describe("Rolling Splits PoC", () => {
  const id = Math.floor(Math.random() * 1000000000);
  const idBuf = Buffer.alloc(4);
  idBuf.writeUInt32LE(id);

  const alice = web3.Keypair.generate();
  const bob = web3.Keypair.generate();
  const carol = web3.Keypair.generate();

  const [split] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("split"), pg.wallet.publicKey.toBuffer(), idBuf],
    pg.program.programId
  );
  const recipientPda = (w) =>
    web3.PublicKey.findProgramAddressSync(
      [Buffer.from("recipient"), split.toBuffer(), w.toBuffer()],
      pg.program.programId
    )[0];

  const sleep = async () => { for (let k = 0; k < 4; k++) { await pg.connection.getSlot(); } };
  const waitFor = async (fn, ok) => {
    for (let i = 0; i < 15; i++) {
      try {
        const v = await fn();
        if (ok(v)) return v;
      } catch (e) {}
      await sleep();
    }
    return await fn();
  };
  const fails = async (p) => {
    try { await p; } catch (e) { return true; }
    return false;
  };
  const deposit = async (lamports) => {
    const before = await pg.connection.getBalance(split);
    const tx = new web3.Transaction().add(
      web3.SystemProgram.transfer({
        fromPubkey: pg.wallet.publicKey,
        toPubkey: split,
        lamports,
      })
    );
    await web3.sendAndConfirmTransaction(pg.connection, tx, [pg.wallet.keypair]);
    await waitFor(() => pg.connection.getBalance(split), (v) => v >= before + lamports);
  };

  it("creates a split", async () => {
    const tx = await pg.program.methods
      .createSplit(id)
      .accounts({
        split,
        authority: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log("create_split tx:", tx);
    const s = await waitFor(
      () => pg.program.account.splitConfig.fetch(split),
      (v) => v.id === id
    );
    assert.equal(s.totalBps, 0);
  }).timeout(60000);

  it("adds a 70% and a 30% recipient", async () => {
    await pg.program.methods
      .addRecipient(7000)
      .accounts({
        split,
        recipient: recipientPda(alice.publicKey),
        wallet: alice.publicKey,
        authority: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    await pg.program.methods
      .addRecipient(3000)
      .accounts({
        split,
        recipient: recipientPda(bob.publicKey),
        wallet: bob.publicKey,
        authority: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    const s = await waitFor(
      () => pg.program.account.splitConfig.fetch(split),
      (v) => v.totalBps === 10000
    );
    assert.equal(s.recipientCount, 2);
  }).timeout(60000);

  it("rejects shares over 100%", async () => {
    const failed = await fails(
      pg.program.methods
        .addRecipient(1)
        .accounts({
          split,
          recipient: recipientPda(carol.publicKey),
          wallet: carol.publicKey,
          authority: pg.wallet.publicKey,
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc()
    );
    assert.ok(failed);
  }).timeout(60000);

  it("refuses a claim before the lock", async () => {
    const failed = await fails(
      pg.program.methods
        .claim()
        .accounts({
          split,
          recipient: recipientPda(alice.publicKey),
          wallet: alice.publicKey,
        })
        .signers([alice])
        .rpc()
    );
    assert.ok(failed);
  }).timeout(60000);

  it("locks the split", async () => {
    const tx = await pg.program.methods
      .lock()
      .accounts({ split, authority: pg.wallet.publicKey })
      .rpc();
    console.log("lock tx:", tx);
    const s = await waitFor(
      () => pg.program.account.splitConfig.fetch(split),
      (v) => v.locked === true
    );
    assert.ok(s.locked);
  }).timeout(60000);

  it("alice claims 70% of a deposit", async () => {
    await deposit(10000000);
    const tx = await pg.program.methods
      .claim()
      .accounts({
        split,
        recipient: recipientPda(alice.publicKey),
        wallet: alice.publicKey,
      })
      .signers([alice])
      .rpc();
    console.log("claim tx:", tx);
    const b = await waitFor(
      () => pg.connection.getBalance(alice.publicKey),
      (v) => v === 7000000
    );
    assert.equal(b, 7000000);
  }).timeout(60000);

  it("bob claims 30% of the same deposit", async () => {
    await pg.program.methods
      .claim()
      .accounts({
        split,
        recipient: recipientPda(bob.publicKey),
        wallet: bob.publicKey,
      })
      .signers([bob])
      .rpc();
    const b = await waitFor(
      () => pg.connection.getBalance(bob.publicKey),
      (v) => v === 3000000
    );
    assert.equal(b, 3000000);
  }).timeout(60000);

  it("refuses a second claim with nothing new", async () => {
    const failed = await fails(
      pg.program.methods
        .claim()
        .accounts({
          split,
          recipient: recipientPda(alice.publicKey),
          wallet: alice.publicKey,
        })
        .signers([alice])
        .rpc()
    );
    assert.ok(failed);
  }).timeout(60000);

  it("pays out late revenue", async () => {
    await deposit(10000000);
    await pg.program.methods
      .claim()
      .accounts({
        split,
        recipient: recipientPda(alice.publicKey),
        wallet: alice.publicKey,
      })
      .signers([alice])
      .rpc();
    const b = await waitFor(
