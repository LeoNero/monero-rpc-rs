use monero_rpc::WalletClient;

async fn create_wallet(
    wallet: &WalletClient,
    password: Option<String>,
    language: String,
) -> anyhow::Result<String> {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    let wallet_name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect();

    wallet
        .create_wallet(wallet_name.clone(), password, language)
        .await
        .map(|_| wallet_name)
}

pub async fn create_wallet_with_password(wallet: &WalletClient, password: &str) -> String {
    create_wallet(wallet, Some(password.to_string()), "English".to_string())
        .await
        .unwrap()
}

pub async fn create_wallet_with_no_password_parameter(wallet: &WalletClient) -> String {
    create_wallet(wallet, None, "English".to_string())
        .await
        .unwrap()
}

pub async fn create_wallet_with_empty_password(wallet: &WalletClient) -> String {
    create_wallet(wallet, Some("".to_string()), "English".to_string())
        .await
        .unwrap()
}

pub async fn create_wallet_error_already_exists(wallet: &WalletClient, wallet_name: &str) {
    let wallet_err = wallet
        .create_wallet(wallet_name.to_string(), None, "English".to_string())
        .await
        .unwrap_err();
    assert_eq!(
        wallet_err.to_string(),
        "Server error: Cannot create wallet. Already exists."
    );
}

pub async fn create_wallet_error_invalid_language(wallet: &WalletClient) {
    let wallet_err = create_wallet(wallet, None, ":thinking_face".to_string())
        .await
        .unwrap_err();
    assert_eq!(
        wallet_err.to_string(),
        "Server error: Unknown language: :thinking_face"
    );
}

pub async fn get_version(wallet: &WalletClient) {
    let version = wallet.get_version().await.unwrap();
    assert_eq!(version, (1, 24));
}
