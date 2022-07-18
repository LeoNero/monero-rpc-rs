use std::str::FromStr;

use monero::{Address, Network};
use monero_rpc::{
    BlockHash, BlockTemplate, GenerateBlocksResponse, HashString, RegtestDaemonClient,
};

use crate::common;

pub async fn get_block_count(regtest: &RegtestDaemonClient, expected_height: u64) {
    let count = regtest.get_block_count().await.unwrap();
    assert_eq!(count.get(), expected_height);
}

pub async fn on_get_block_hash(
    regtest: &RegtestDaemonClient,
    height: u64,
    expected_hash: BlockHash,
) {
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

pub async fn get_block_template(
    regtest: &RegtestDaemonClient,
    address: Address,
    reserve_size: u64,
    expected_block_template: BlockTemplate,
) {
    let mut res_block_template = regtest
        .get_block_template(address, reserve_size)
        .await
        .unwrap();

    // this field is not deterministic
    res_block_template.blockhashing_blob = HashString(vec![]);
    // this field is not deterministic
    res_block_template.blocktemplate_blob = HashString(vec![]);

    assert_eq!(res_block_template, expected_block_template);
}

pub async fn get_block_template_error_invalid_reserve_size(
    regtest: &RegtestDaemonClient,
    address: Address,
) {
    let res_err = regtest.get_block_template(address, 256).await.unwrap_err();
    assert_eq!(
        res_err.to_string(),
        "Server error: Too big reserved size, maximum 255"
    );
}

pub async fn get_block_template_error_invalid_address(regtest: &RegtestDaemonClient) {
    let key_pair_1 = common::get_keypair_1();
    let address_testnet = Address::from_keypair(Network::Testnet, &key_pair_1);
    let res_err = regtest
        .get_block_template(address_testnet, 10)
        .await
        .unwrap_err();
    assert_eq!(
        res_err.to_string(),
        "Server error: Failed to parse wallet address"
    );
}

pub async fn submit_block(regtest: &RegtestDaemonClient, block_template_blob: HashString<Vec<u8>>) {
    let start_block_count = regtest.get_block_count().await.unwrap().get();
    regtest
        .submit_block(block_template_blob.to_string())
        .await
        .unwrap();
    assert_eq!(
        start_block_count + 1,
        regtest.get_block_count().await.unwrap().get()
    );

    // submitting same blob again returns success but does not increase block count
    regtest
        .submit_block(block_template_blob.to_string())
        .await
        .unwrap();
    assert_eq!(
        start_block_count + 1,
        regtest.get_block_count().await.unwrap().get()
    );
}

pub async fn submit_block_error_wrong_block_blob(regtest: &RegtestDaemonClient) {
    let block_template_blob = "0123456789";

    let res_err = regtest
        .submit_block(block_template_blob.to_string())
        .await
        .unwrap_err();
    assert_eq!(res_err.to_string(), "Server error: Wrong block blob");
}

pub async fn submit_block_error_block_not_accepted(regtest: &RegtestDaemonClient) {
    let block_template_blob = "0707e6bdfedc053771512f1bc27c62731ae9e8f2443db64ce742f4e57f5cf8d393de28551e441a0000000002fb830a01ffbf830a018cfe88bee283060274c0aae2ef5730e680308d9c00b6da59187ad0352efe3c71d36eeeb28782f29f2501bd56b952c3ddc3e350c2631d3a5086cac172c56893831228b17de296ff4669de020200000000";
    let res_err = regtest
        .submit_block(block_template_blob.to_string())
        .await
        .unwrap_err();
    assert_eq!(res_err.to_string(), "Server error: Block not accepted");
}
