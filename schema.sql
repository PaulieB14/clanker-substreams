-- Clanker Substreams SQL Schema
-- For use with substreams-sink-sql

-- Clanker tokens registry
CREATE TABLE IF NOT EXISTS tokens (
    address VARCHAR(42) PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    log_index BIGINT NOT NULL,
    admin VARCHAR(42) NOT NULL,
    name TEXT NOT NULL,
    symbol VARCHAR(32) NOT NULL,
    image TEXT,
    metadata TEXT,
    context TEXT,
    pool_id VARCHAR(66) NOT NULL,
    pool_hook VARCHAR(42) NOT NULL,
    paired_token VARCHAR(42) NOT NULL,
    starting_tick INT NOT NULL,
    locker VARCHAR(42) NOT NULL,
    mev_module VARCHAR(42),
    extensions_supply NUMERIC,
    msg_sender VARCHAR(42) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Fee claims
CREATE TABLE IF NOT EXISTS fee_claims (
    id VARCHAR(128) PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    log_index BIGINT NOT NULL,
    token VARCHAR(42) NOT NULL,
    recipient VARCHAR(42) NOT NULL,
    amount NUMERIC NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Extensions triggered (airdrops, presales, etc.)
CREATE TABLE IF NOT EXISTS extensions_triggered (
    id VARCHAR(128) PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    log_index BIGINT NOT NULL,
    extension VARCHAR(42) NOT NULL,
    extension_supply NUMERIC NOT NULL,
    msg_value NUMERIC NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Token metadata updates
CREATE TABLE IF NOT EXISTS metadata_updates (
    id VARCHAR(128) PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    log_index BIGINT NOT NULL,
    token_address VARCHAR(42) NOT NULL,
    update_type VARCHAR(16) NOT NULL,
    new_value TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Token verifications
CREATE TABLE IF NOT EXISTS verifications (
    id VARCHAR(128) PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    log_index BIGINT NOT NULL,
    token_address VARCHAR(42) NOT NULL,
    admin VARCHAR(42) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Token transfers (ERC20)
CREATE TABLE IF NOT EXISTS transfers (
    id VARCHAR(128) PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,
    block_number BIGINT NOT NULL,
    block_timestamp BIGINT NOT NULL,
    log_index BIGINT NOT NULL,
    token_address VARCHAR(42) NOT NULL,
    from_address VARCHAR(42) NOT NULL,
    to_address VARCHAR(42) NOT NULL,
    amount NUMERIC NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_tokens_block ON tokens(block_number);
CREATE INDEX IF NOT EXISTS idx_tokens_admin ON tokens(admin);
CREATE INDEX IF NOT EXISTS idx_tokens_symbol ON tokens(symbol);

CREATE INDEX IF NOT EXISTS idx_fee_claims_token ON fee_claims(token);
CREATE INDEX IF NOT EXISTS idx_fee_claims_recipient ON fee_claims(recipient);
CREATE INDEX IF NOT EXISTS idx_fee_claims_block ON fee_claims(block_number);

CREATE INDEX IF NOT EXISTS idx_extensions_block ON extensions_triggered(block_number);
CREATE INDEX IF NOT EXISTS idx_extensions_extension ON extensions_triggered(extension);

CREATE INDEX IF NOT EXISTS idx_metadata_token ON metadata_updates(token_address);
CREATE INDEX IF NOT EXISTS idx_metadata_block ON metadata_updates(block_number);

CREATE INDEX IF NOT EXISTS idx_verifications_token ON verifications(token_address);

CREATE INDEX IF NOT EXISTS idx_transfers_token ON transfers(token_address);
CREATE INDEX IF NOT EXISTS idx_transfers_from ON transfers(from_address);
CREATE INDEX IF NOT EXISTS idx_transfers_to ON transfers(to_address);
CREATE INDEX IF NOT EXISTS idx_transfers_block ON transfers(block_number);
