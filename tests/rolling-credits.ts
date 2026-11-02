describe("Rolling Credits PoC", () => {
  const slug = "pilot-" + Date.now().toString().slice(-6);
  const contributor = web3.Keypair.generate();
  const stranger = web3.Keypair.generate();

  const [production] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("production"), pg.wallet.publicKey.toBuffer(), Buffer.from(slug)],
    pg.program.programId
  );
  const [credit] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("credit"), production.toBuffer(), contributor.publicKey.toBuffer()],
    pg.program.programId
  );

  const sleep = async () => { for (let k = 0; k < 4; k++) { await pg.connection.getSlot(); } };
  const waitFor = async (fn, ok) => {
    for (let i = 0; i < 15; i++) {
      try {
        const v = await fn();
        if (ok(v)) return v;
      } catch (e) {}
      await sleep(2000);
    }
    return await fn();
  };

  it("creates a production", async () => {
    const tx = await pg.program.methods
      .createProduction(slug, "Rolling Credits Pilot")
      .accounts({
        production,
        owner: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log("create_production tx:", tx);
    const p = await waitFor(
      () => pg.program.account.production.fetch(production),
      (v) => v.slug === slug
    );
    assert.equal(p.slug, slug);
  }).timeout(60000);

  it("proposes a credit", async () => {
    const tx = await pg.program.methods
      .proposeCredit("Gaffer")
      .accounts({
        production,
        credit,
        contributor: contributor.publicKey,
        owner: pg.wallet.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log("propose_credit tx:", tx);
    const c = await waitFor(
      () => pg.program.account.credit.fetch(credit),
      (v) => v.role === "Gaffer"
    );
    assert.equal(c.role, "Gaffer");
  }).timeout(60000);

  it("refuses an accept from the wrong wallet", async () => {
    let failed = false;
    try {
      await pg.program.methods
        .acceptCredit()
        .accounts({ credit, contributor: stranger.publicKey })
        .signers([stranger])
        .rpc();
    } catch (err) {
      failed = true;
    }
    assert.ok(failed);
  }).timeout(60000);

  it("contributor accepts and the credit is verified", async () => {
    const tx = await pg.program.methods
      .acceptCredit()
      .accounts({ credit, contributor: contributor.publicKey })
      .signers([contributor])
      .rpc();
    console.log("accept_credit tx:", tx);
    const c = await waitFor(
      () => pg.program.account.credit.fetch(credit),
      (v) => !!v.status.verified
    );
    assert.ok(c.status.verified);
  }).timeout(60000);

  it("can't accept the same credit twice", async () => {
    let failed = false;
    try {
      await pg.program.methods
        .acceptCredit()
        .accounts({ credit, contributor: contributor.publicKey })
        .signers([contributor])
        .rpc();
    } catch (err) {
      failed = true;
    }
    assert.ok(failed);
  }).timeout(60000);
});
