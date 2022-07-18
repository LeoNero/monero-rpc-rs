use std::str::FromStr;

use monero::{Address, Network};
use monero_rpc::{BlockHash, GenerateBlocksResponse, RegtestDaemonClient};

pub async fn get_block_count(regtest: &RegtestDaemonClient, expected_height: u64) {
    let count = regtest.get_block_count().await.unwrap();
    assert_eq!(count.get(), expected_height);
}

pub async fn on_get_block_hash(regtest: &RegtestDaemonClient, height: u64, expected_hash: BlockHash) {
    let block_hash = regtest.on_get_block_hash(height).await.unwrap();
    assert_eq!(block_hash, expected_hash);
}

pub async fn on_get_block_hash_error_invalid_height(regtest: &RegtestDaemonClient, height: u64) {
    let block_hash = regtest.on_get_block_hash(height).await.unwrap_err();
    assert_eq!(
        block_hash.to_string(),
        format!("Invalid height {height} supplied.")
    );
}

async fn get_expected_height_returned_by_generate_blocks(
    start_block_count: u64,
    amount_of_blocks: u64,
) -> u64 {
    let height = start_block_count - 1;
    height + amount_of_blocks
}

pub async fn generate_blocks(
    regtest: &RegtestDaemonClient,
    amount_of_blocks: u64,
    wallet_address: Address,
) -> GenerateBlocksResponse {
    let start_block_count = regtest.get_block_count().await.unwrap().get();

    let res = regtest
        .generate_blocks(amount_of_blocks, wallet_address)
        .await
        .unwrap();
    let expected_height =
        get_expected_height_returned_by_generate_blocks(start_block_count, amount_of_blocks).await;
    assert_eq!(res.height, expected_height);
    assert!(res.blocks.is_some());

    let final_block_count = regtest.get_block_count().await.unwrap().get();
    assert_eq!(start_block_count + amount_of_blocks, final_block_count);

    res
}

// This is to demonstrate that, if `amount_of_blocks` is zero, then the RPC returns success even if
// the address is wrong
pub async fn generate_blocks_zero_blocks(regtest: &RegtestDaemonClient, wallet_address: Address) {
    if let Network::Mainnet = wallet_address.network {
        panic!("generate_blocks_zero_blocks only accepts an address that is not in the Mainnet/Regtest format.")
    }

    let start_block_count = regtest.get_block_count().await.unwrap().get();

    let amount_of_blocks = 0;
    let res = regtest
        .generate_blocks(amount_of_blocks, wallet_address)
        .await
        .unwrap();

    let expected_height =
        get_expected_height_returned_by_generate_blocks(start_block_count, amount_of_blocks).await;

    assert_eq!(res.height, expected_height + 1);
    assert!(res.blocks.is_none());
    assert_eq!(
        start_block_count + amount_of_blocks,
        regtest.get_block_count().await.unwrap().get(),
    );
}

// We are on regtest, but the address used in this function is **not** a regtest address
pub async fn generate_blocks_error_invalid_address(
    regtest: &RegtestDaemonClient,
    wallet_address: Address,
) {
    if let Network::Mainnet = wallet_address.network {
        panic!("generate_blocks_error_invalid_address only accepts an address that is not in the Mainnet/Regtest format.")
    }

    let err = regtest
        .generate_blocks(100, wallet_address)
        .await
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Server error: Failed to parse wallet address"
    );
}
