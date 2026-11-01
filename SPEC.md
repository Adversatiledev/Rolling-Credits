# Rolling Credits Specification

**Version 0.1, draft.** Nothing here is implemented yet. Expect names and field sizes to change during the build. Open questions are listed at the end.

## Overview

Two Anchor programs:

- **`credits`** records productions and mutually signed contributor credits.
- **`splits`** holds tokens in a vault and lets recipients claim their share.

They don't depend on each other. `splits` can optionally read a verified `Credit` account when adding a recipient.

## `credits` program

### Accounts

**Production**
PDA seeds: `["production", owner, slug]`

| Field | Type | Notes |
| --- | --- | --- |
| owner | Pubkey | Can propose and revoke credits |
| slug | String (max 32) | Unique per owner |
| title | String (max 64) | Display name |
| credit_count | u32 | Number of credits ever proposed |
| created_at | i64 | Unix timestamp |
| bump | u8 | |

**Credit**
PDA seeds: `["credit", production, contributor]`

| Field | Type | Notes |
| --- | --- | --- |
| production | Pubkey | Parent production |
| contributor | Pubkey | The credited wallet |
| role | String (max 48) | For example "Gaffer" or "Sound Recordist" |
| status | enum | Proposed, Verified, Declined, Revoked |
| owner_revoke | bool | Owner has signed a revoke request |
| contributor_revoke | bool | Contributor has signed a revoke request |
| proposed_at | i64 | |
| verified_at | i64 | Zero until accepted |
| bump | u8 | |

One credit per contributor per production in v0.1. Multiple roles go in the `role` string. See open questions.

### Instructions

| Instruction | Signer | Effect |
| --- | --- | --- |
| `create_production(slug, title)` | owner | Creates a Production |
| `propose_credit(role)` | owner | Creates a Credit in Proposed status |
| `accept_credit` | contributor | Proposed to Verified |
| `decline_credit` | contributor | Proposed to Declined |
| `request_revoke` | owner or contributor | Sets that party's revoke flag. When both are set, status becomes Revoked |
| `cancel_revoke` | owner or contributor | Clears that party's flag |

A Verified credit's role can't be edited. To change it, revoke and re-propose.

### Events

`ProductionCreated`, `CreditProposed`, `CreditVerified`, `CreditDeclined`, `CreditRevoked`. The indexer builds the adoption dashboard from these.

## `splits` program

### Accounts

**SplitConfig**
PDA seeds: `["split", authority, id]`

| Field | Type | Notes |
| --- | --- | --- |
| authority | Pubkey | Can edit recipients until locked |
| id | u64 | Lets one authority run many splits |
| mint | Pubkey | Token this split pays out |
| vault | Pubkey | Associated token account owned by this PDA |
| production | Option<Pubkey> | Optional link to a `credits` Production |
| recipient_count | u16 | |
| total_bps | u16 | Must equal 10,000 to lock |
| locked | bool | |
| total_claimed | u64 | Running total paid out |
| bump | u8 | |

**Recipient**
PDA seeds: `["recipient", split, wallet]`

| Field | Type | Notes |
| --- | --- | --- |
| split | Pubkey | |
| wallet | Pubkey | Who gets paid |
| bps | u16 | Share in basis points |
| claimed | u64 | Running total this recipient has claimed |
| bump | u8 | |

Each recipient is its own account, so there's no fixed cap on team size.

### Instructions

| Instruction | Signer | Effect |
| --- | --- | --- |
| `create_split(id, mint, production?)` | authority | Creates SplitConfig and vault |
| `add_recipient(wallet, bps)` | authority | Adds a recipient. Rejected once locked |
| `add_recipient_from_credit(bps)` | authority | Same, but requires a Verified Credit on the linked production |
| `update_recipient(bps)` / `remove_recipient` | authority | Rejected once locked |
| `lock` | authority | Requires total_bps = 10,000. Irreversible |
| `claim` | recipient wallet | Pays out what's owed. Requires locked |

There is no deposit instruction. Anyone sends tokens straight to the vault address, so revenue can arrive from an exchange, a payment processor or another program without integrating anything.

### Claim maths

Shares are fixed once locked, so the program never needs to track deposits:

```
total_received = vault.amount + config.total_claimed
entitled       = floor(total_received * recipient.bps / 10_000)
owed           = entitled - recipient.claimed
```

On claim, the program transfers `owed`, then adds it to both `recipient.claimed` and `config.total_claimed`. Intermediate maths uses u128.

Why pull, not push: a push payout to 80 recipients won't fit in one transaction and needs someone to crank it. Pull keeps every claim the same size and cost, whatever the team size, and nobody pays fees on anyone else's behalf.

Rounding: flooring leaves at most a few base units per recipient in the vault. A dust sweep is an open question.

### Events

`SplitCreated`, `RecipientAdded`, `RecipientUpdated`, `SplitLocked`, `Claimed`.

## Security considerations

- Locking is irreversible and checked on every claim, so recipients can't be swapped after money arrives.
- Vault authority is the SplitConfig PDA. No one, including the authority, can withdraw except through `claim`.
- Token accounts are checked against the config's mint and vault on every instruction.
- All arithmetic is checked. No silent overflow.
- An independent security review is a roadmap milestone, published in full before mainnet.

## SDK (planned)

TypeScript on `@solana/kit`, wrapping every instruction, plus React Native hooks for Mobile Wallet Adapter. Target developer flow:

1. Create a production and propose credits.
2. Contributors accept from any wallet, including on mobile.
3. Create a split from verified credits and lock it.
4. Share the vault address wherever revenue arrives.
5. Contributors claim whenever they like.

## Open questions

- Multiple roles per contributor per production: one Credit with a list, or one Credit per role?
- Token-2022 extensions: transfer fees and transfer hooks change the amount received. Support them, or reject those mints in v0.1?
- Dust: let the authority sweep rounding remainders after a set period, or leave them?
- Should a split be closable once fully claimed, to recover rent?
- Should `propose_credit` be open to delegates, such as a production coordinator, not just the owner?

Feedback on any of these is welcome. Open an issue.
