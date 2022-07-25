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

// TODO wallet.get_payments success
// TODO wallet.get_payments error
// TODO wallet.get_bulk_payments success
// TODO wallet.get_bulk_payments error
// TODO wallet.sweep_all success
// TODO wallet.sweep_all error
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
    //     main_tests::non_empty_blockchain_test().await;
    // });
    // let handle3 = tokio::spawn(async {
    //     main_tests::basic_daemon_rpc_test().await;
    // });
    //
    // let res = tokio::try_join!(handle1, handle2, handle3);
    // res.unwrap();

    main_tests::all_clients_interaction_test().await;
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
