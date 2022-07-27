use std::str::FromStr;

use crate::common::helpers;
use monero::cryptonote::hash::Hash;
use monero_rpc::{HashString, TransactionsResponse};

pub async fn test() {
    let (_, daemon_rpc, _) = helpers::setup_monero();

    // empty `txs_hashes`
    let expected_transactions_response = TransactionsResponse {
        credits: 0,
        status: "OK".to_string(),
        top_hash: "".to_string(),
        untrusted: false,
        missed_tx: None,
        txs: None,
        txs_as_hex: None,
        txs_as_json: None,
    };

    helpers::daemon_rpc::get_transactions(
        &daemon_rpc,
        vec![],
        expected_transactions_response.clone(),
    )
    .await;

    // valid hash, but non existent transaction in `txs_hashes`
    let tx_hash =
        Hash::from_str("d6e48158472848e6687173a91ae6eebfa3e1d778e65252ee99d7515d63090408").unwrap();
    let expected_transactions_response = TransactionsResponse {
        credits: 0,
        status: "OK".to_string(),
        top_hash: "".to_string(),
        untrusted: false,
        missed_tx: Some(vec![HashString(tx_hash)]),
        txs: None,
        txs_as_hex: None,
        txs_as_json: None,
    };

    helpers::daemon_rpc::get_transactions(
        &daemon_rpc,
        vec![tx_hash],
        expected_transactions_response,
    )
    .await;
}