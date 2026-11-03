# Rolling Credits

Open-source Solana programs and SDK for contributor credits and revenue splits.

Rolling Credits records who worked on something, and routes the money it earns back to them. It was designed for film and TV crew, and it works for any team that pays more than one contributor.

## Status

**Devnet proofs of concept.** Two programs are deployed on Solana devnet, each with passing tests and a published IDL. Everything else is still planned.

This is devnet only. It is unaudited and should not hold real funds.

| Program | Devnet program ID | Explorer |
| --- | --- | --- |
| Credits | `EEXBANe3p9Y6CRZ1PENKkdxrEBHsGhYi3VoqxPHHFuEk` | https://explorer.solana.com/address/EEXBANe3p9Y6CRZ1PENKkdxrEBHsGhYi3VoqxPHHFuEk?cluster=devnet |
| Splits | `EsYnYU2o5tzeWR78Z48jRgbAqWQHPubokY33ecBZctUN` | https://explorer.solana.com/address/EsYnYU2o5tzeWR78Z48jRgbAqWQHPubokY33ecBZctUN?cluster=devnet |

| Component | Status |
| --- | --- |
| `credits` program: create production, propose credit, accept credit | Devnet PoC, 5 tests passing |
| `credits` program: decline, revoke | Planned |
| `splits` program: create split, add recipient, lock, claim (SOL only) | Devnet PoC, 9 tests passing |
| `splits` program: SPL tokens such as USDC | Planned |
| TypeScript SDK | Planned |
| React Native hooks (Mobile Wallet Adapter) | Planned |
| Public indexer and adoption dashboard | Planned |
| Security review | Planned |
| Ourobor.OS integration | Planned |

## What the proofs of concept do

**Credits**

- An owner creates a production onchain.
- The owner proposes a credit to a contributor's wallet, with a role such as Gaffer.
- The contributor accepts with their own signature, and the credit becomes verified.
- A wallet that isn't the credited contributor can't accept it, and a credit can't be accepted twice.

**Splits**

- An authority creates a split and adds recipients with shares in basis points, for example 7,000 and 3,000.
- Shares can't add up to more than 10,000, and can't be changed once the split is locked.
- Locking needs shares to add up to exactly 10,000.
- Anyone can send SOL to the split's address. Each recipient claims their share when they like.
- Claims are refused before the lock, and a repeat claim with nothing new is refused.
- Revenue that arrives after a claim is paid out correctly on the next claim.

The splits proof of concept differs from SPEC.md in one way. It holds SOL in the split account itself, where the full design uses a vault for SPL tokens.

## Repo layout

- `programs/rolling-credits/src/lib.rs`: the credits program.
- `programs/rolling-splits/src/lib.rs`: the splits program.
- `tests/rolling-credits.ts`: five tests, written for Solana Playground.
- `tests/rolling-splits.ts`: nine tests, written for Solana Playground.
- `SPEC.md`: the full design and open questions.

## Run the tests

The tests run in Solana Playground for now. Moving to a standard `anchor test` setup is on the roadmap.

1. Open beta.solpg.io and create a new Anchor (Rust) project.
2. Paste in the program's `lib.rs` and its test file.
3. Build, then deploy to devnet with a funded Playground wallet.
4. Type `test` in the terminal.

Expected results are five passing tests for credits and nine for splits.

## The problem

Every app that pays a team has to answer two questions: who did what, and who gets paid how much. Today each team writes its own escrow, pays for its own audit, and invents a credit format nothing else can read.

Creative work makes this worse. A film crew is 20 to 80 people. Money arrives from sales, licences, screenings and streams, often years later, in amounts too small to split by hand. Existing onchain creator splits are tied to NFT sales and capped at a handful of creators.

## How it works

**Credits.** A production owner proposes a credit to a contributor's wallet. The contributor accepts with their own signature. The credit is now verified, readable by any app or program.

**Splits.** A split holds each recipient's share in basis points, summing to 10,000. Anyone can pay into it. Recipients claim what they're owed, whenever they like. Shares lock once set, so earned money can't be redirected.

The two programs are independent. Use credits without splits, or splits without credits, or link them so verified credits seed the split.

Full design: [SPEC.md](SPEC.md).

## Why Solana

Splitting a $10 licence fee across 60 people means 60 payments of a few cents each. On most chains the fees eat the payment. On Solana they don't. Mobile Wallet Adapter and Seeker's Seed Vault let contributors sign and claim from their phones, which is where crew actually live.

## Roadmap

1. **Finish credits.** Decline and revoke, a standard Anchor test setup, and a devnet trial with external users.
2. **Splits with SPL tokens.** USDC support, claim maths tested to 80 recipients, independent security review published, mainnet deploy.
3. **SDK, mobile hooks and indexer.** SDK on npm, end-to-end test on Seeker, public dashboard.
4. **Maintenance.** Six months of active support: issues, fixes, compatibility, monthly changelog.

## First integrator

[Ourobor.OS](https://adversatileartists.com), a gamified operating system for independent creative founders, by Adversatile Artists, is the planned first integrator. It is not integrated yet. Ourobor.OS is a separate, commercial product. Rolling Credits is open infrastructure anyone can use.

## Contributing

Issues and ideas are welcome. If you're building something that pays contributors, open an issue and tell us what you'd need.

## Contact

Jordan Williams, Adversatile Artists. jordan@adversatileartists.com

## Licence

Apache-2.0. See [LICENSE](LICENSE).
