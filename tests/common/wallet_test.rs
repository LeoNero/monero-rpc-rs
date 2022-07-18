use monero_rpc::WalletClient;

pub async fn open_wallet_error_file_not_exists(wallet: &WalletClient) {
    let opened_wallet = wallet
        .open_wallet("wallet_name_that_doesnt_exist".to_string(), None)
        .await
        .unwrap_err();
    assert_eq!(
        opened_wallet.to_string(),
        "Server error: Failed to open wallet"
    );
}
