describe("Credits decline and revoke", () => {
  const slug = "rev-" + Date.now().toString().slice(-6);
  const alice = web3.Keypair.generate();
  const bob = web3.Keypair.generate();
  const stranger = web3.Keypair.generate();

  const [production] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("production"), pg.wallet.publicKey.toBuffer(), Buffer.from(slug)],
    pg.program.programId
  );
  const creditPda = (w) =>
    web3.PublicKey.findProgramAddressSync(
      [Buffer.from("credit"), production.toBuffer(), w.toBuffer()],
      pg.program.programId
    )[0];
  const aliceCredit = creditPda(alice.publicKey);
  const bobCredit = creditPda(bob.publicKey);

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
  const credit = (pda) => pg.program.account.credit.fetch(pda);

  it("sets up a production with two proposed credits", async () => {
    await pg.program.methods
      .createProduction(slug, "Revoke Test")
      .accounts({
        production,
        owner: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    await pg.program.methods
      .proposeCredit("Gaffer")
      .accounts({
        production,
        credit: aliceCredit,
        contributor: alice.publicKey,
        owner: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    await pg.program.methods
      .proposeCredit("Editor")
      .accounts({
        production,
        credit: bobCredit,
        contributor: bob.publicKey,
        owner: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    const c = await waitFor(() => credit(bobCredit), (v) => v.role === "Editor");
    assert.equal(c.role, "Editor");
  }).timeout(90000);

  it("alice declines her credit", async () => {
    await pg.program.methods
      .declineCredit()
      .accounts({ credit: aliceCredit, contributor: alice.publicKey })
      .signers([alice])
      .rpc();
    const c = await waitFor(() => credit(aliceCredit), (v) => !!v.status.declined);
    assert.ok(c.status.declined);
  }).timeout(60000);

  it("a declined credit can't be accepted", async () => {
    const failed = await fails(
      pg.program.methods
        .acceptCredit()
        .accounts({ credit: aliceCredit, contributor: alice.publicKey })
        .signers([alice])
        .rpc()
    );
    assert.ok(failed);
  }).timeout(60000);

  it("bob accepts, so his credit is verified", async () => {
    await pg.program.methods
      .acceptCredit()
      .accounts({ credit: bobCredit, contributor: bob.publicKey })
      .signers([bob])
      .rpc();
    const c = await waitFor(() => credit(bobCredit), (v) => !!v.status.verified);
    assert.ok(c.status.verified);
  }).timeout(60000);

  it("a stranger can't request a revoke", async () => {
    const failed = await fails(
      pg.program.methods
        .requestRevoke()
        .accounts({ credit: bobCredit, production, signer: stranger.publicKey })
        .signers([stranger])
        .rpc()
    );
    assert.ok(failed);
  }).timeout(60000);

  it("the owner asks to revoke, but the credit stays verified", async () => {
    await pg.program.methods
      .requestRevoke()
      .accounts({ credit: bobCredit, production, signer: pg.wallet.publicKey })
      .rpc();
    const c = await waitFor(() => credit(bobCredit), (v) => v.ownerRevoke === true);
    assert.ok(c.status.verified);
    assert.equal(c.contributorRevoke, false);
  }).timeout(60000);

  it("the owner can cancel the request", async () => {
    await pg.program.methods
      .cancelRevoke()
      .accounts({ credit: bobCredit, production, signer: pg.wallet.publicKey })
      .rpc();
    const c = await waitFor(() => credit(bobCredit), (v) => v.ownerRevoke === false);
    assert.equal(c.ownerRevoke, false);
  }).timeout(60000);

  it("both sides sign, so the credit is revoked", async () => {
    await pg.program.methods
      .requestRevoke()
      .accounts({ credit: bobCredit, production, signer: pg.wallet.publicKey })
      .rpc();
    await pg.program.methods
      .requestRevoke()
      .accounts({ credit: bobCredit, production, signer: bob.publicKey })
      .signers([bob])
      .rpc();
    const c = await waitFor(() => credit(bobCredit), (v) => !!v.status.revoked);
    assert.ok(c.status.revoked);
  }).timeout(90000);
});
