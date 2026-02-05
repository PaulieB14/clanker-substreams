mod abi;
mod pb;

use abi::clanker_factory::events as factory_events;
use abi::clanker_token::events as token_events;
use pb::clanker::v1::{
    ClankerEvents, ExtensionTriggered, FeeClaim, Token, TokenCreated, TokenMetadataUpdate,
    TokenTransfer, TokenTransfers, TokenVerified,
};
use std::str::FromStr;
use substreams::errors::Error;
use substreams::scalar::BigInt;
use substreams::store::{StoreAdd, StoreAddBigInt, StoreAddInt64, StoreGet, StoreGetProto, StoreNew, StoreSet, StoreSetProto};
use substreams::Hex;
use substreams_database_change::pb::sf::substreams::sink::database::v1::DatabaseChanges;
use substreams_database_change::tables::Tables;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

/// Clanker Factory contract address (can be overridden via params)
const DEFAULT_CLANKER_FACTORY: &str = "20dd04c17afd5c9a8b3f2cdacaa8ee7907385bef";

/// Parse the factory address from params
fn parse_factory_address(params: &str) -> Vec<u8> {
    let factory_hex = params
        .split(',')
        .find_map(|p| {
            let parts: Vec<&str> = p.split('=').collect();
            if parts.len() == 2 && parts[0].trim() == "clanker_factory" {
                Some(parts[1].trim().trim_start_matches("0x"))
            } else {
                None
            }
        })
        .unwrap_or(DEFAULT_CLANKER_FACTORY);

    hex::decode(factory_hex).unwrap_or_else(|_| hex::decode(DEFAULT_CLANKER_FACTORY).unwrap())
}

/// Map Clanker factory events (TokenCreated, FeeClaims, etc.)
#[substreams::handlers::map]
pub fn map_clanker_events(params: String, block: Block) -> Result<ClankerEvents, Error> {
    let factory_address = parse_factory_address(&params);
    let mut events = ClankerEvents::default();

    let block_number = block.number;
    let block_timestamp = block.timestamp_seconds();

    for trx in block.transactions() {
        for (log, _call) in trx.logs_with_calls() {
            // Check if log is from factory contract
            let is_factory = log.address == factory_address.as_slice();

            // Process factory events
            if is_factory {
                // TokenCreated event
                if let Some(event) = factory_events::TokenCreated::match_and_decode(log) {
                    let extensions: Vec<String> = event
                        .extensions
                        .iter()
                        .map(|e| Hex::encode(e))
                        .collect();

                    events.token_created.push(TokenCreated {
                        tx_hash: Hex::encode(&trx.hash),
                        block_number,
                        block_timestamp,
                        log_index: log.index as u64,
                        token_address: Hex::encode(&event.token_address),
                        token_admin: Hex::encode(&event.token_admin),
                        token_name: event.token_name.clone(),
                        token_symbol: event.token_symbol.clone(),
                        token_image: event.token_image.clone(),
                        token_metadata: event.token_metadata.clone(),
                        token_context: event.token_context.clone(),
                        pool_id: Hex::encode(&event.pool_id),
                        pool_hook: Hex::encode(&event.pool_hook),
                        paired_token: Hex::encode(&event.paired_token),
                        starting_tick: event.starting_tick.to_string().parse::<i32>().unwrap_or(0),
                        locker: Hex::encode(&event.locker),
                        mev_module: Hex::encode(&event.mev_module),
                        extensions_supply: event.extensions_supply.to_u64(),
                        extensions,
                        msg_sender: Hex::encode(&event.msg_sender),
                    });
                }

                // ClaimTeamFees event
                if let Some(event) = factory_events::ClaimTeamFees::match_and_decode(log) {
                    events.fee_claims.push(FeeClaim {
                        tx_hash: Hex::encode(&trx.hash),
                        block_number,
                        block_timestamp,
                        log_index: log.index as u64,
                        token: Hex::encode(&event.token),
                        recipient: Hex::encode(&event.recipient),
                        amount: event.amount.to_string(),
                    });
                }

                // ExtensionTriggered event
                if let Some(event) = factory_events::ExtensionTriggered::match_and_decode(log) {
                    events.extensions_triggered.push(ExtensionTriggered {
                        tx_hash: Hex::encode(&trx.hash),
                        block_number,
                        block_timestamp,
                        log_index: log.index as u64,
                        extension: Hex::encode(&event.extension),
                        extension_supply: event.extension_supply.to_string(),
                        msg_value: event.msg_value.to_string(),
                    });
                }
            }

            // Token-level events (UpdateImage, UpdateMetadata, Verified)
            // These come from individual token contracts
            if let Some(event) = token_events::UpdateImage::match_and_decode(log) {
                events.metadata_updates.push(TokenMetadataUpdate {
                    tx_hash: Hex::encode(&trx.hash),
                    block_number,
                    block_timestamp,
                    log_index: log.index as u64,
                    token_address: Hex::encode(&log.address),
                    update_type: "image".to_string(),
                    new_value: event.image.clone(),
                });
            }

            if let Some(event) = token_events::UpdateMetadata::match_and_decode(log) {
                events.metadata_updates.push(TokenMetadataUpdate {
                    tx_hash: Hex::encode(&trx.hash),
                    block_number,
                    block_timestamp,
                    log_index: log.index as u64,
                    token_address: Hex::encode(&log.address),
                    update_type: "metadata".to_string(),
                    new_value: event.metadata.clone(),
                });
            }

            if let Some(event) = token_events::Verified::match_and_decode(log) {
                events.verifications.push(TokenVerified {
                    tx_hash: Hex::encode(&trx.hash),
                    block_number,
                    block_timestamp,
                    log_index: log.index as u64,
                    token_address: Hex::encode(&event.token),
                    admin: Hex::encode(&event.admin),
                });
            }
        }
    }

    Ok(events)
}

/// Store tokens in a registry for lookups
#[substreams::handlers::store]
pub fn store_tokens(events: ClankerEvents, store: StoreSetProto<Token>) {
    for token_created in events.token_created {
        let key = format!("token:{}", token_created.token_address);
        store.set(
            0,
            &key,
            &Token {
                address: token_created.token_address.clone(),
                name: token_created.token_name,
                symbol: token_created.token_symbol,
                admin: token_created.token_admin,
                image: token_created.token_image,
                pool_id: token_created.pool_id,
                paired_token: token_created.paired_token,
                created_at_block: token_created.block_number,
                created_at_timestamp: token_created.block_timestamp,
            },
        );
    }
}

/// Map ERC20 transfers for Clanker tokens only
#[substreams::handlers::map]
pub fn map_token_transfers(
    block: Block,
    store: StoreGetProto<Token>,
) -> Result<TokenTransfers, Error> {
    let mut transfers = TokenTransfers::default();

    let block_number = block.number;
    let block_timestamp = block.timestamp_seconds();

    for trx in block.transactions() {
        for (log, _call) in trx.logs_with_calls() {
            // Check if this is a known Clanker token
            let token_key = format!("token:{}", Hex::encode(&log.address));
            if store.get_last(&token_key).is_none() {
                continue;
            }

            // Transfer event
            if let Some(event) = token_events::Transfer::match_and_decode(log) {
                transfers.transfers.push(TokenTransfer {
                    tx_hash: Hex::encode(&trx.hash),
                    block_number,
                    block_timestamp,
                    log_index: log.index as u64,
                    token_address: Hex::encode(&log.address),
                    from: Hex::encode(&event.from),
                    to: Hex::encode(&event.to),
                    amount: event.value.to_string(),
                });
            }
        }
    }

    Ok(transfers)
}

/// Output to database sink
#[substreams::handlers::map]
pub fn db_out(
    events: ClankerEvents,
    transfers: TokenTransfers,
) -> Result<DatabaseChanges, Error> {
    let mut tables = Tables::new();

    // Insert token creations
    for token in &events.token_created {
        tables
            .create_row("tokens", &token.token_address)
            .set("tx_hash", &token.tx_hash)
            .set("block_number", token.block_number)
            .set("block_timestamp", token.block_timestamp)
            .set("log_index", token.log_index)
            .set("admin", &token.token_admin)
            .set("name", &token.token_name)
            .set("symbol", &token.token_symbol)
            .set("image", &token.token_image)
            .set("metadata", &token.token_metadata)
            .set("context", &token.token_context)
            .set("pool_id", &token.pool_id)
            .set("pool_hook", &token.pool_hook)
            .set("paired_token", &token.paired_token)
            .set("starting_tick", token.starting_tick)
            .set("locker", &token.locker)
            .set("mev_module", &token.mev_module)
            .set("extensions_supply", token.extensions_supply)
            .set("msg_sender", &token.msg_sender);
    }

    // Insert fee claims
    for fee in &events.fee_claims {
        let pk = format!("{}-{}", fee.tx_hash, fee.log_index);
        tables
            .create_row("fee_claims", &pk)
            .set("tx_hash", &fee.tx_hash)
            .set("block_number", fee.block_number)
            .set("block_timestamp", fee.block_timestamp)
            .set("log_index", fee.log_index)
            .set("token", &fee.token)
            .set("recipient", &fee.recipient)
            .set("amount", &fee.amount);
    }

    // Insert extensions triggered
    for ext in &events.extensions_triggered {
        let pk = format!("{}-{}", ext.tx_hash, ext.log_index);
        tables
            .create_row("extensions_triggered", &pk)
            .set("tx_hash", &ext.tx_hash)
            .set("block_number", ext.block_number)
            .set("block_timestamp", ext.block_timestamp)
            .set("log_index", ext.log_index)
            .set("extension", &ext.extension)
            .set("extension_supply", &ext.extension_supply)
            .set("msg_value", &ext.msg_value);
    }

    // Insert metadata updates
    for update in &events.metadata_updates {
        let pk = format!("{}-{}", update.tx_hash, update.log_index);
        tables
            .create_row("metadata_updates", &pk)
            .set("tx_hash", &update.tx_hash)
            .set("block_number", update.block_number)
            .set("block_timestamp", update.block_timestamp)
            .set("log_index", update.log_index)
            .set("token_address", &update.token_address)
            .set("update_type", &update.update_type)
            .set("new_value", &update.new_value);
    }

    // Insert verifications
    for v in &events.verifications {
        let pk = format!("{}-{}", v.tx_hash, v.log_index);
        tables
            .create_row("verifications", &pk)
            .set("tx_hash", &v.tx_hash)
            .set("block_number", v.block_number)
            .set("block_timestamp", v.block_timestamp)
            .set("log_index", v.log_index)
            .set("token_address", &v.token_address)
            .set("admin", &v.admin);
    }

    // Insert transfers
    for transfer in &transfers.transfers {
        let pk = format!("{}-{}", transfer.tx_hash, transfer.log_index);
        tables
            .create_row("transfers", &pk)
            .set("tx_hash", &transfer.tx_hash)
            .set("block_number", transfer.block_number)
            .set("block_timestamp", transfer.block_timestamp)
            .set("log_index", transfer.log_index)
            .set("token_address", &transfer.token_address)
            .set("from_address", &transfer.from)
            .set("to_address", &transfer.to)
            .set("amount", &transfer.amount);
    }

    Ok(tables.to_database_changes())
}

// ============================================================================
// Additional Stores for Analytics
// ============================================================================

/// Store transfer volume per token (accumulates BigInt)
#[substreams::handlers::store]
pub fn store_token_volume(transfers: TokenTransfers, store: StoreAddBigInt) {
    for transfer in &transfers.transfers {
        let key = format!("volume:{}", transfer.token_address);
        if let Ok(amount) = BigInt::from_str(&transfer.amount) {
            store.add(0, &key, amount);
        }
    }
}

/// Store transfer counts per token
#[substreams::handlers::store]
pub fn store_token_transfer_counts(transfers: TokenTransfers, store: StoreAddInt64) {
    for transfer in &transfers.transfers {
        let key = format!("transfers:{}", transfer.token_address);
        store.add(0, &key, 1);
    }
}

/// Store total fees claimed per creator (recipient)
#[substreams::handlers::store]
pub fn store_creator_fees(events: ClankerEvents, store: StoreAddBigInt) {
    for fee in &events.fee_claims {
        let key = format!("fees:{}", fee.recipient);
        if let Ok(amount) = BigInt::from_str(&fee.amount) {
            store.add(0, &key, amount);
        }
    }
}

/// Store token counts per creator
#[substreams::handlers::store]
pub fn store_creator_token_counts(events: ClankerEvents, store: StoreAddInt64) {
    for token in &events.token_created {
        let key = format!("tokens:{}", token.token_admin);
        store.add(0, &key, 1);
    }
}

