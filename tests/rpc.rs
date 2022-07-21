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

mod common;
use common::main_tests;

// all below with valid transactions
// decode_as_json = None, prune = false
// decode_as_json = None, prune = true
// decode_as_json = false, prune = false
// decode_as_json = false, prune = true
// decode_as_json = true, prune = false
// decode_as_json = true, prune = true
// TODO daemon_rpc.get_transactions

// TODO wallet.get_height success
// TODO wallet.get_height error
// TODO wallet.get_balance success
// TODO wallet.get_balance error
// TODO wallet.refresh success
// TODO wallet.refresh error
// TODO wallet.get_payments success
// TODO wallet.get_payments error
// TODO wallet.get_bulk_payments success
// TODO wallet.get_bulk_payments error
// TODO wallet.query_key success
// TODO wallet.query_key error
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

#[tokio::test]
async fn main_functional_test() {
    // TODO uncomment all

    // run those tests functions concurrently since the state one changes does not affect the state
    // the other one interacts with.
    // let handle1 = tokio::spawn(main_tests::basic_wallet_test());
    // let handle2 = tokio::spawn(async {
    //     main_tests::empty_blockchain_test().await;
    //     main_tests::non_empty_blockchain().await;
    // });
    let handle3 = tokio::spawn(async {
        main_tests::basic_daemon_rpc_test().await;
    });

    let res = tokio::try_join!(/* handle1 */ /* handle2,*/ handle3);
    res.unwrap();

    // main_tests::all_clients_interaction_test().await;
}

// makes sure the Rust code in the readme works
#[tokio::test]
async fn readme_test() {
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

/*
* TODO
#[tokio::test]
async fn functional_wallet_test() {
    wallet.get_height().await.unwrap();

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
} */
