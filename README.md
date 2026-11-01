# Rolling Credits

Open-source Solana programs and SDK for contributor credits and revenue splits.

Rolling Credits records who worked on something, and routes the money it earns back to them. It was designed for film and TV crew, and it works for any team that pays more than one contributor.

## Status

**Pre-build.** Nothing is deployed yet. This repo holds the specification and roadmap. Code lands here, in public, from the first commit.

| Component | Status |
| --- | --- |
| `credits` program | Planned |
| `splits` program | Planned |
| TypeScript SDK | Planned |
| React Native hooks (Mobile Wallet Adapter) | Planned |
| Public indexer and adoption dashboard | Planned |
| Security review | Planned |

## The problem

Every app that pays a team has to answer two questions: who did what, and who gets paid how much. Today each team writes its own escrow, pays for its own audit, and invents a credit format nothing else can read.

Creative work makes this worse. A film crew is 20 to 80 people. Money arrives from sales, licences, screenings and streams, often years later, in amounts too small to split by hand. Existing onchain creator splits are tied to NFT sales and capped at a handful of creators.

## How it works

**Credits.** A production owner proposes a credit to a contributor's wallet. The contributor accepts with their own signature. The credit is now verified, readable by any app or program. Revoking it needs both signatures, so neither side can quietly erase the other.

**Splits.** A split holds each recipient's share in basis points, summing to 10,000. Its vault accepts any SPL token, typically USDC, from anyone. Recipients claim what they're owed, whenever they like. Shares lock once revenue arrives, so earned money can't be redirected.

The two programs are independent. Use credits without splits, or splits without credits, or link them so verified credits seed the split.

Full design: [SPEC.md](SPEC.md).

## Why Solana

Splitting a $10 licence fee across 60 people means 60 payments of a few cents each. On most chains the fees eat the payment. On Solana they don't. Mobile Wallet Adapter and Seeker's Seed Vault let contributors sign and claim from their phones, which is where crew actually live.

## Roadmap

1. **Credits program beta.** Tests in CI, then devnet trial with external users.
2. **Splits program beta.** Claim maths tested to 80 recipients, independent security review published, mainnet deploy.
3. **SDK, mobile hooks and indexer.** SDK on npm, end-to-end test on Seeker, public dashboard.
4. **Maintenance.** Six months of active support: issues, fixes, compatibility, monthly changelog.

## First integrator

[Ourobor.OS](https://adversatileartists.com), a gamified operating system for independent creative founders, by Adversatile Artists. Ourobor.OS is a separate, commercial product. Rolling Credits is open infrastructure anyone can use.

## Contributing

Issues and ideas are welcome now, even before the first line of code. If you're building something that pays contributors, open an issue and tell us what you'd need.

## Contact

Jordan Williams, Adversatile Artists. jordan@adversatileartists.com

## Licence

Apache-2.0. See [LICENSE](LICENSE).
