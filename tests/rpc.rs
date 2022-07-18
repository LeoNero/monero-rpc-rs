// Copyright 2019-2022 Artem Vorotnikov and Monero Rust Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono::{DateTime, NaiveDateTime, Utc};
use hex::FromHex;
use monero::{Address, Amount, KeyPair, Network, PrivateKey};
use monero_rpc::{BlockHash, BlockHeaderResponse, BlockTemplate, HashString};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

mod common;
use common::{daemon_rpc_test, regtest_test, wallet_test};

fn setup_monero() -> (
    monero_rpc::RegtestDaemonClient,
    monero_rpc::DaemonRpcClient,
    monero_rpc::WalletClient,
) {
    let dhost = env::var("MONERO_DAEMON_HOST").unwrap_or_else(|_| "localhost".into());

    let rpc_client = monero_rpc::RpcClient::new(format!("http://{}:18081", dhost));
    let daemon = rpc_client.daemon();
    let regtest = daemon.regtest();

    let rpc_client = monero_rpc::RpcClient::new(format!("http://{}:18081", dhost));
    let daemon_rpc = rpc_client.daemon_rpc();

    let whost = env::var("MONERO_WALLET_HOST_1").unwrap_or_else(|_| "localhost".into());
    let rpc_client = monero_rpc::RpcClient::new(format!("http://{}:18083", whost));
    let wallet = rpc_client.wallet();

    (regtest, daemon_rpc, wallet)
}

// With no transactions in array, with transactions in array, with non-existent txs, with invalid
// txs hashes
// TODO daemon_rpc.get_transactions success decode_as_json=true
// TODO daemon_rpc.get_transactions success decode_as_json=false
// TODO daemon_rpc.get_transactions success decode_as_json=None
// TODO daemon_rpc.get_transactions success prune=true
// TODO daemon_rpc.get_transactions success prune=false
// TODO daemon_rpc.get_transactions success prune=None
// TODO daemon_rpc.get_transactions error txs_hashes
//
// TODO wallet.generate_from_keys success
// TODO wallet.generate_from_keys error
// TODO wallet.create_wallet success
// TODO wallet.create_wallet error
// TODO wallet.open_wallet success
// TODO wallet.open_wallet error -> wrong password
// TODO wallet.close_wallet success
// TODO wallet.close_wallet error
// TODO wallet.get_balance success
// TODO wallet.get_balance error
// TODO wallet.get_address success
// TODO wallet.get_address error
// TODO wallet.get_address_index success
// TODO wallet.get_address_index error
// TODO wallet.create_address success
// TODO wallet.create_address error
// TODO wallet.label_address success
// TODO wallet.label_address error
// TODO wallet.refresh success
// TODO wallet.refresh error
// TODO wallet.get_accounts success
// TODO wallet.get_accounts error
// TODO wallet.get_payments success
// TODO wallet.get_payments error
// TODO wallet.get_bulk_payments success
// TODO wallet.get_bulk_payments error
// TODO wallet.query_key success
// TODO wallet.query_key error
// TODO wallet.get_height success
// TODO wallet.get_height error
// TODO wallet.sweep_all success
// TODO wallet.sweep_all error
// TODO wallet.relay_tx success
// TODO wallet.relay_tx error
// TODO wallet.transfer success
// TODO wallet.transfer error
// TODO wallet.sign_transfer success
// TODO wallet.sign_transfer error
// TODO wallet.submit_transfer success
// TODO wallet.submit_transfer error
// TODO wallet.incoming_transfers success
// TODO wallet.incoming_transfers error
// TODO wallet.get_transfers success
// TODO wallet.get_transfers error
// TODO wallet.get_transfer success
// TODO wallet.get_transfer error
// TODO wallet.export_key_images success
// TODO wallet.export_key_images error
// TODO wallet.import_key_images success
// TODO wallet.import_key_images error
// TODO wallet.check_tx_key success
// TODO wallet.check_tx_key error
// TODO wallet.get_version success
// TODO wallet.get_version error

#[tokio::test]
async fn main_functional_test() {
    // run those two functions concurrently since the state one changes does not affect the other
    let handle1 = tokio::spawn(async {
        basic_wallet_test().await;
    });
    let handle2 = tokio::spawn(async {
        empty_blockchain().await;
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    non_empty_blockchain().await;
}

async fn basic_wallet_test() {
    let (regtest, daemon_rpc, wallet) = setup_monero();

    wallet_test::open_wallet_error_file_not_exists(&wallet).await;
}

async fn empty_blockchain() {
    let (regtest, daemon_rpc, wallet) = setup_monero();

    let genesis_block_hash = common::get_genesis_block_hash();

    regtest_test::get_block_count(&regtest, 1).await;
    regtest_test::on_get_block_hash_error_invalid_height(&regtest, 10).await;
    regtest_test::on_get_block_hash(&regtest, 0, genesis_block_hash).await;

    let key_pair_1 = common::get_keypair_1();
    let address_1 = Address::from_keypair(Network::Mainnet, &key_pair_1);

    regtest_test::get_block_template(
        &regtest,
        address_1,
        10,
        BlockTemplate {
            // this field is not deterministic, so set it to empty vec
            blockhashing_blob: HashString(vec![]),
            // this field is not deterministic, so set it to empty vec
            blocktemplate_blob: HashString(vec![]),
            difficulty: 1,
            expected_reward: 35184338534400,
            height: 1,
            prev_hash: HashString(genesis_block_hash),
            reserved_offset: 126,
            untrusted: false,
        },
    )
    .await;
    regtest_test::get_block_template_error_invalid_reserve_size(&regtest, address_1).await;
    regtest_test::get_block_template_error_invalid_address(&regtest).await;

    let genesis_block_header = BlockHeaderResponse {
        block_size: 80,
        depth: 0,
        difficulty: 1,
        hash: genesis_block_hash,
        height: 0,
        major_version: 1,
        minor_version: 0,
        nonce: 10000,
        num_txes: 0,
        orphan_status: false,
        prev_hash: BlockHash::zero(),
        reward: 17592186044415,
        // this is used in the assert, since it is the genesis block
        timestamp: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
    };

    regtest_test::get_last_block_header(&regtest, genesis_block_header.clone()).await;
    regtest_test::get_block_header_from_block_hash(
        &regtest,
        genesis_block_hash,
        genesis_block_header.clone(),
    )
    .await;

    regtest_test::get_block_header_from_block_hash_error_not_found(
        &regtest,
        BlockHash::from_slice(&[0; 32]),
    )
    .await;
    regtest_test::get_block_header_from_block_hash_error_not_found(
        &regtest,
        BlockHash::from_slice(&[42; 32]),
    )
    .await;

    let current_top_block_height = regtest.get_block_count().await.unwrap().get() - 1;
    regtest_test::get_block_header_at_height(&regtest, 0, genesis_block_header.clone()).await;
    regtest_test::get_block_header_at_height_error(&regtest, 10, current_top_block_height).await;

    regtest_test::get_block_headers_range(&regtest, 0..=0, vec![genesis_block_header]).await;
    regtest_test::get_block_headers_range_error(&regtest, 0..=4).await;
    regtest_test::get_block_headers_range_error(&regtest, 2..=4).await;
    regtest_test::get_block_headers_range_error(&regtest, 4..=0).await;
}

async fn non_empty_blockchain() {
    let (regtest, daemon_rpc, wallet) = setup_monero();

    let key_pair_1 = common::get_keypair_1();

    let address_testnet = Address::from_keypair(Network::Testnet, &key_pair_1);
    regtest_test::generate_blocks_error_invalid_address(&regtest, address_testnet).await;
    regtest_test::generate_blocks_zero_blocks(&regtest, address_testnet).await;

    let address_1 = Address::from_keypair(Network::Mainnet, &key_pair_1);
    let generate_blocks_res = regtest_test::generate_blocks(&regtest, 10, address_1).await;

    let last_two_added_blocks: Vec<BlockHash> = generate_blocks_res
        .blocks
        .unwrap()
        .into_iter()
        .rev()
        .take(2)
        .collect();
    let last_added_block_hash = last_two_added_blocks[0];
    let last_but_one_added_block_hash = last_two_added_blocks[1];

    regtest_test::on_get_block_hash_error_invalid_height(&regtest, generate_blocks_res.height + 1)
        .await;
    regtest_test::on_get_block_hash(&regtest, generate_blocks_res.height, last_added_block_hash)
        .await;

    let last_added_block_header = BlockHeaderResponse {
        block_size: 85,
        depth: 0,
        difficulty: 1,
        hash: last_added_block_hash,
        height: regtest.get_block_count().await.unwrap().get() - 1,
        major_version: 16,
        minor_version: 16,
        nonce: 0,
        num_txes: 0,
        orphan_status: false,
        prev_hash: last_but_one_added_block_hash,
        reward: 35183734559807,
        // this is not used in the assert, so use any date
        timestamp: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
    };
    regtest_test::get_last_block_header(&regtest, last_added_block_header.clone()).await;
    regtest_test::get_block_header_from_block_hash(
        &regtest,
        last_added_block_hash,
        last_added_block_header.clone(),
    )
    .await;

    let current_top_block_height = regtest.get_block_count().await.unwrap().get() - 1;

    regtest_test::get_block_header_at_height(&regtest, 10, last_added_block_header).await;
    regtest_test::get_block_header_at_height_error(&regtest, u64::MAX, current_top_block_height)
        .await;

    let last_block_header = regtest
        .get_block_header(monero_rpc::GetBlockHeaderSelector::Height(
            current_top_block_height,
        ))
        .await
        .unwrap();
    let last_but_one_block_header = regtest
        .get_block_header(monero_rpc::GetBlockHeaderSelector::Height(
            current_top_block_height - 1,
        ))
        .await
        .unwrap();
    regtest_test::get_block_headers_range(
        &regtest,
        9..=10,
        vec![last_but_one_block_header, last_block_header],
    )
    .await;

    let block_template = regtest.get_block_template(address_1, 0).await.unwrap();
    regtest_test::submit_block(&regtest, block_template.blocktemplate_blob).await;
    regtest_test::submit_block_error_wrong_block_blob(&regtest).await;
    regtest_test::submit_block_error_block_not_accepted(&regtest).await;
}

/*
* TODO
#[tokio::test]
async fn function_readme_test() {
    assert!(false);
    let tx_id = "7c50844eced8ab78a8f26a126fbc1f731134e0ae3e6f9ba0f205f98c1426ff60".to_string();
    let daemon_client =
        monero_rpc::RpcClient::new("http://node.monerooutreach.org:18081".to_string());
    let daemon = daemon_client.daemon_rpc();
    let mut fixed_hash: [u8; 32] = [0; 32];
    hex::decode_to_slice(tx_id, &mut fixed_hash).unwrap();
    let tx = daemon
        .get_transactions(vec![fixed_hash.into()], Some(true), Some(true))
        .await;
    println!("tx {:?}", tx);
    println!(
        "unlock time: {:?}",
        serde_json::from_str::<monero_rpc::JsonTransaction>(&tx.unwrap().txs_as_json.unwrap()[0])
    );
}

* TODO
#[tokio::test]
async fn functional_daemon_test() {
    assert!(false);
    let addr_str = "4AdUndXHHZ6cfufTMvppY6JwXNouMBzSkbLYfpAV5Usx3skxNgYeYTRj5UzqtReoS44qo9mtmXCqY45DJ852K5Jv2684Rge";
    let (regtest, _) = setup_monero();
    let address = Address::from_str(addr_str).unwrap();
    regtest.get_block_template(address, 60).await.unwrap();
    regtest.get_block_count().await.unwrap();

    let a = regtest.on_get_block_hash(1).await.unwrap();
    println!("{:?}", a);

    regtest
        .get_block_header(monero_rpc::GetBlockHeaderSelector::Last)
        .await
        .unwrap();
    regtest.generate_blocks(4, address).await.unwrap();
    regtest
        .get_block_headers_range(std::ops::RangeInclusive::new(1, 2))
        .await
        .unwrap();
}

* TODO
#[tokio::test]
async fn functional_wallet_test() {
    assert!(false);
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    let spend_wallet_name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect();
    let view_wallet_name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect();

    let (regtest, wallet) = setup_monero();
    match wallet
        .create_wallet(spend_wallet_name.clone(), None, "English".to_string())
        .await
    {
        Ok(_) => {}
        Err(err) => {
            assert_eq!(
                format!("{}", err),
                "Server error: Cannot create wallet. Already exists."
            );
        }
    }
    wallet
        .open_wallet(spend_wallet_name.clone(), None)
        .await
        .unwrap();

    // test closing the wallet again
    wallet.close_wallet().await.unwrap();
    assert_eq!(
        format!(
            "{}",
            wallet.get_address(0, Some(vec![0])).await.err().unwrap()
        ),
        "Server error: No wallet file".to_string()
    );

    wallet
        .open_wallet(spend_wallet_name.clone(), None)
        .await
        .unwrap();
    wallet.get_balance(1, Some(vec![0])).await.unwrap();
    let address = wallet.get_address(0, Some(vec![0])).await.unwrap().address;
    wallet.get_address_index(address).await.unwrap();
    wallet
        .create_address(0, Some("new_label".to_string()))
        .await
        .unwrap();
    wallet
        .label_address(0, 0, "other_label".to_string())
        .await
        .unwrap();
    wallet.get_accounts(None).await.unwrap();
    wallet.get_height().await.unwrap();
    wallet.get_version().await.unwrap();

    regtest.generate_blocks(500, address).await.unwrap();
    wallet.refresh(Some(0)).await.unwrap();

    let mut destination: HashMap<Address, Amount> = HashMap::new();
    destination.insert(address, Amount::from_xmr(0.00001).unwrap());

    let transfer_options = monero_rpc::TransferOptions {
        account_index: None,
        subaddr_indices: None,
        mixin: Some(10),
        ring_size: Some(11),
        unlock_time: Some(0),
        payment_id: None,
        do_not_relay: Some(true),
    };

    let transfer_data = wallet
        .transfer(
            destination,
            monero_rpc::TransferPriority::Default,
            transfer_options,
        )
        .await
        .unwrap();

    wallet
        .relay_tx(transfer_data.tx_metadata.to_string())
        .await
        .unwrap();

    match wallet
        .check_tx_key(transfer_data.tx_hash.0, transfer_data.tx_key.0, address)
        .await
    {
        Ok(_) => {}
        Err(err) => {
            let err_string = format!("{}", err);
            assert!(
                err_string == "invalid value: integer `0`, expected a nonzero u64".to_string()
                    || err_string == "expected a non-zero value"
            );
        }
    }

    wallet.export_key_images().await.unwrap();

    wallet
        .query_key(monero_rpc::PrivateKeyType::Spend)
        .await
        .unwrap();
    let viewkey = wallet
        .query_key(monero_rpc::PrivateKeyType::View)
        .await
        .unwrap();

    match wallet
        .generate_from_keys(monero_rpc::GenerateFromKeysArgs {
            restore_height: Some(0),
            filename: view_wallet_name.clone(),
            address,
            spendkey: None,
            viewkey,
            password: "".to_string(),
            autosave_current: None,
        })
        .await
    {
        Ok(_) => {}
        Err(err) => {
            assert_eq!(format!("{}", err), "Server error: Wallet already exists.");
        }
    }

    wallet
        .open_wallet(view_wallet_name.clone(), None)
        .await
        .unwrap();
    wallet.export_key_images().await.unwrap();

    wallet.refresh(Some(0)).await.unwrap();

    wallet
        .incoming_transfers(monero_rpc::TransferType::All, Some(0), Some(vec![0, 1, 2]))
        .await
        .unwrap();

    use monero_rpc::{GetTransfersCategory, GetTransfersSelector};

    let mut category_selector: HashMap<GetTransfersCategory, bool> = HashMap::new();
    category_selector.insert(GetTransfersCategory::In, true);
    category_selector.insert(GetTransfersCategory::Out, true);
    category_selector.insert(GetTransfersCategory::Pending, true);
    category_selector.insert(GetTransfersCategory::Pool, true);

    let selector = GetTransfersSelector {
        category_selector,
        subaddr_indices: None,
        account_index: None,
        block_height_filter: Some(monero_rpc::BlockHeightFilter {
            min_height: Some(0),
            max_height: None,
        }),
    };

    wallet.get_transfers(selector).await.unwrap();

    let mut destination: HashMap<Address, Amount> = HashMap::new();
    destination.insert(address, Amount::from_xmr(0.00001).unwrap());

    let transfer_options = monero_rpc::TransferOptions {
        account_index: Some(0),
        subaddr_indices: Some(vec![0]),
        mixin: Some(10),
        ring_size: Some(11),
        unlock_time: Some(0),
        payment_id: None,
        do_not_relay: Some(true),
    };

    let transfer_data = wallet
        .transfer(
            destination,
            monero_rpc::TransferPriority::Default,
            transfer_options,
        )
        .await
        .unwrap();

    wallet
        .open_wallet(spend_wallet_name.clone(), None)
        .await
        .unwrap();
    wallet.refresh(Some(0)).await.unwrap();

    let sweep_args = monero_rpc::SweepAllArgs {
        address,
        account_index: 0,
        subaddr_indices: None,
        priority: monero_rpc::TransferPriority::Default,
        mixin: 10,
        ring_size: 11,
        unlock_time: 0,
        get_tx_keys: None,
        below_amount: None,
        do_not_relay: None,
        get_tx_hex: None,
        get_tx_metadata: None,
    };
    wallet.sweep_all(sweep_args).await.unwrap();

    let res = wallet
        .sign_transfer(transfer_data.unsigned_txset.0)
        .await
        .unwrap();
    println!("res: {:?}", res);
} */
