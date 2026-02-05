# Clanker Substreams

High-performance blockchain indexer for [Clanker](https://clanker.world) token factory on Base, built with [Substreams](https://substreams.streamingfast.io).

## What is Clanker?

Clanker is a set of audited smart contracts on Base that create token markets rewarding token creators. Tokens are deployed via Farcaster, web interface, SDK, or direct contract interaction.

## What This Indexes

- **Token Launches** - All new Clanker tokens with full metadata
- **Fee Claims** - Creator and team reward distributions
- **Extensions** - Airdrop, presale, and other extension triggers
- **Metadata Updates** - Token image and metadata changes
- **Verifications** - Token verification events
- **Transfers** - ERC20 transfers for all Clanker tokens

## Prerequisites

- [Rust](https://rustup.rs/) with `wasm32-unknown-unknown` target
- [Substreams CLI](https://substreams.streamingfast.io/getting-started/installing-the-cli)
- [Buf CLI](https://buf.build/docs/installation)
- [The Graph Market API Key](https://thegraph.market/auth/signup)

## Quick Start

### 1. Install Dependencies

```bash
# Install Rust wasm target
rustup target add wasm32-unknown-unknown

# Install substreams CLI (macOS)
brew install streamingfast/tap/substreams

# Authenticate
substreams auth
```

### 2. Build

```bash
# Generate protobuf code and compile to WASM
substreams build
```

### 3. Run

```bash
# Test with a small block range
substreams run -e base map_clanker_events -s 22520000 -t +1000

# Run with JSON output
substreams run -e base map_clanker_events -s 22520000 -t +1000 -o jsonl
```

### 4. Deploy to SQL Database

```bash
# Create the database tables
psql -d your_database -f schema.sql

# Run the SQL sink
substreams-sink-sql run \
  base \
  "postgresql://user:pass@localhost:5432/clanker?sslmode=disable" \
  substreams.yaml \
  db_out
```

## Modules

| Module | Type | Description |
|--------|------|-------------|
| `map_clanker_events` | Map | Extracts factory events (TokenCreated, FeeClaims, etc.) |
| `store_tokens` | Store | Maintains registry of all Clanker tokens |
| `map_token_transfers` | Map | ERC20 transfers for known Clanker tokens |
| `db_out` | Map | Database sink output (PostgreSQL/ClickHouse) |

## Configuration

The Clanker factory address can be configured via params:

```bash
substreams run -e base map_clanker_events \
  -p map_clanker_events="clanker_factory=0x20dd04c17afd5c9a8b3f2cdacaa8ee7907385bef" \
  -s 22520000 -t +1000
```

## Contract Addresses (Base v4.1.0)

| Contract | Address |
|----------|---------|
| Clanker Factory | `0x20DD04c17AFD5c9a8b3f2cdacaa8Ee7907385BEF` |
| ClankerHookDynamicFeeV2 | `0xd60D6B218b30aF607a6Fd77dD956F9baC9B50d00` |
| ClankerHookStaticFeeV2 | `0xb4429d62f8f3bFFb98CdB9574E23499A8ED08Cc` |
| ClankerSniperAuctionV2 | `0xebB25BB797D82CB78E1bc70406b13233c0854413` |
| ClankerAirdropV2 | `0xf652B3610D75D81871bf96DB50825d9af28391E0` |

## Project Structure

```
clanker-substreams/
├── substreams.yaml          # Manifest
├── schema.sql               # SQL schema for sink
├── Cargo.toml               # Rust dependencies
├── build.rs                 # ABI code generation
├── abi/
│   ├── clanker_factory.json # Factory ABI
│   └── clanker_token.json   # Token ABI
├── proto/
│   └── clanker.proto        # Protobuf schemas
└── src/
    ├── lib.rs               # Module implementations
    ├── abi/
    │   └── mod.rs           # Generated ABI bindings
    └── pb/                  # Generated protobuf code (auto)
```

## Example Queries

### Get all tokens launched in last 24 hours
```sql
SELECT name, symbol, admin, block_timestamp
FROM tokens
WHERE block_timestamp > EXTRACT(EPOCH FROM NOW() - INTERVAL '24 hours')
ORDER BY block_timestamp DESC;
```

### Get top creators by token count
```sql
SELECT admin, COUNT(*) as token_count
FROM tokens
GROUP BY admin
ORDER BY token_count DESC
LIMIT 20;
```

### Get fee claims for a token
```sql
SELECT recipient, amount, block_timestamp
FROM fee_claims
WHERE token = '0x...'
ORDER BY block_timestamp DESC;
```

## Resources

- [Clanker Documentation](https://clanker.gitbook.io/clanker-documentation)
- [Clanker SDK](https://github.com/clanker-devco/clanker-sdk)
- [Substreams Documentation](https://substreams.streamingfast.io)
- [StreamingFast Discord](https://discord.gg/streamingfast)

## License

MIT
