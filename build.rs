fn main() {
    // Generate Rust bindings for Clanker Factory ABI
    substreams_ethereum::Abigen::new("ClankerFactory", "abi/clanker_factory.json")
        .expect("Failed to load Clanker Factory ABI")
        .generate()
        .expect("Failed to generate Clanker Factory bindings")
        .write_to_file("src/abi/clanker_factory.rs")
        .expect("Failed to write Clanker Factory bindings");

    // Generate Rust bindings for ClankerToken ABI
    substreams_ethereum::Abigen::new("ClankerToken", "abi/clanker_token.json")
        .expect("Failed to load ClankerToken ABI")
        .generate()
        .expect("Failed to generate ClankerToken bindings")
        .write_to_file("src/abi/clanker_token.rs")
        .expect("Failed to write ClankerToken bindings");

    // Generate Rust bindings for ClankerAirdrop ABI
    substreams_ethereum::Abigen::new("ClankerAirdrop", "abi/clanker_airdrop.json")
        .expect("Failed to load ClankerAirdrop ABI")
        .generate()
        .expect("Failed to generate ClankerAirdrop bindings")
        .write_to_file("src/abi/clanker_airdrop.rs")
        .expect("Failed to write ClankerAirdrop bindings");

    // Generate Rust bindings for ClankerAuction ABI
    substreams_ethereum::Abigen::new("ClankerAuction", "abi/clanker_auction.json")
        .expect("Failed to load ClankerAuction ABI")
        .generate()
        .expect("Failed to generate ClankerAuction bindings")
        .write_to_file("src/abi/clanker_auction.rs")
        .expect("Failed to write ClankerAuction bindings");
}
