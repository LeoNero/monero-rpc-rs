use monero::cryptonote::hash::Hash;
use monero_rpc::{DaemonRpcClient, TransactionsResponse};

pub async fn get_transactions(
    daemon_rpc: &DaemonRpcClient,
    txs_hashes: Vec<Hash>,
    expected_transactions_response: TransactionsResponse,
) {
    let transactions_response = daemon_rpc.get_transactions(txs_hashes, None, None).await.unwrap();
    assert_eq!(transactions_response, expected_transactions_response);
}
