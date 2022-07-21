use std::str::FromStr;

use monero::Address;
use monero_rpc::{
    AddressData, GenerateFromKeysArgs, GetAccountsData, WalletClient, WalletCreation,
};

fn get_random_name() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect()
}

pub async fn get_version(wallet: &WalletClient) {
    let version = wallet.get_version().await.unwrap();
    assert_eq!(version, (1, 24));
}

async fn create_wallet(
    wallet: &WalletClient,
    password: Option<String>,
    language: String,
) -> anyhow::Result<String> {
    let wallet_name: String = get_random_name();

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

pub async fn close_wallet(wallet: &WalletClient) {
    wallet.close_wallet().await.unwrap();
}

pub async fn close_wallet_error_no_wallet_file(wallet: &WalletClient) {
    let err = wallet.close_wallet().await.unwrap_err();
    assert_eq!(err.to_string(), "Server error: No wallet file");
}

pub async fn open_wallet_with_password(wallet: &WalletClient, filename: &str, password: &str) {
    wallet
        .open_wallet(filename.to_string(), Some(password.to_string()))
        .await
        .unwrap();
}

pub async fn open_wallet_with_no_or_empty_password(wallet: &WalletClient, filename: &str) {
    // if wallet has no password, both calls should work
    wallet
        .open_wallet(filename.to_string(), Some("".to_string()))
        .await
        .unwrap();
    wallet
        .open_wallet(filename.to_string(), None)
        .await
        .unwrap();
}

pub async fn open_wallet_error_filename_invalid(wallet: &WalletClient, filename: &str) {
    let err = wallet
        .open_wallet(filename.to_string(), None)
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "Server error: Failed to open wallet");
}

pub async fn open_wallet_error_wrong_password(
    wallet: &WalletClient,
    filename: &str,
    password: Option<String>,
) {
    let err = wallet
        .open_wallet(filename.to_string(), password)
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "Server error: Failed to open wallet");
}

pub async fn generate_from_keys(
    wallet: &WalletClient,
    mut args: GenerateFromKeysArgs,
) -> (String, WalletCreation) {
    let filename = get_random_name();

    args.filename = filename.clone();

    let expected_info = if let Some(_) = args.spendkey {
        "Wallet has been generated successfully."
    } else {
        "Watch-only wallet has been generated successfully."
    };
    let wallet_creation = wallet.generate_from_keys(args).await.unwrap();
    assert_eq!(wallet_creation.info, expected_info);

    (filename, wallet_creation)
}

pub async fn generate_from_keys_error_filename_already_exists(
    wallet: &WalletClient,
    args: GenerateFromKeysArgs,
) {
    let wallet_creation_err = wallet.generate_from_keys(args).await.unwrap_err();
    assert_eq!(
        wallet_creation_err.to_string(),
        "Server error: Wallet already exists."
    );
}

pub async fn generate_from_keys_error_invalid_address(
    wallet: &WalletClient,
    args: GenerateFromKeysArgs,
) {
    let wallet_creation_err = wallet.generate_from_keys(args).await.unwrap_err();
    assert_eq!(
        wallet_creation_err.to_string(),
        "Server error: Failed to parse public address"
    );
}

pub async fn get_address(
    wallet: &WalletClient,
    account: u64,
    addresses: Option<Vec<u64>>,
    expected_res: AddressData,
) {
    let addresses = wallet.get_address(account, addresses).await.unwrap();
    assert_eq!(addresses, expected_res);
}

pub async fn get_address_error_no_wallet_file(wallet: &WalletClient) {
    let get_address_err = wallet.get_address(0, None).await.unwrap_err();
    assert_eq!(get_address_err.to_string(), "Server error: No wallet file");
}

pub async fn get_address_error_invalid_account_index(wallet: &WalletClient, account: u64) {
    let get_address_err = wallet.get_address(account, None).await.unwrap_err();
    assert_eq!(
        get_address_err.to_string(),
        "Server error: account index is out of bound"
    );
}

pub async fn get_address_error_invalid_address_index(
    wallet: &WalletClient,
    account: u64,
    addresses: Option<Vec<u64>>,
) {
    let get_address_err = wallet.get_address(account, addresses).await.unwrap_err();
    assert_eq!(
        get_address_err.to_string(),
        "Server error: address index is out of bound"
    );
}

pub async fn get_address_index(
    wallet: &WalletClient,
    address: Address,
    expected_index: (u64, u64),
) {
    let index = wallet.get_address_index(address).await.unwrap();
    assert_eq!(index, expected_index);
}

pub async fn get_address_index_error_address_from_another_wallet(
    wallet: &WalletClient,
    address: Address,
) {
    let index_err = wallet.get_address_index(address).await.unwrap_err();
    assert_eq!(
        index_err.to_string(),
        "Server error: Address doesn't belong to the wallet"
    );
}

pub async fn get_address_index_error_invalid_address(wallet: &WalletClient, address: Address) {
    let index_err = wallet.get_address_index(address).await.unwrap_err();
    assert_eq!(index_err.to_string(), "Server error: Invalid address");
}

pub async fn create_address(
    wallet: &WalletClient,
    account_index: u64,
    label: Option<String>,
    expected_res: (Address, u64),
) -> (Address, u64) {
    let address_created = wallet.create_address(account_index, label).await.unwrap();
    assert_eq!(address_created, expected_res);
    address_created
}

pub async fn create_address_error_invalid_account_index(wallet: &WalletClient, account_index: u64) {
    let create_address_err = wallet
        .create_address(account_index, None)
        .await
        .unwrap_err();
    assert_eq!(
        create_address_err.to_string(),
        "Server error: account index is out of bound"
    );
}

pub async fn label_address(
    wallet: &WalletClient,
    account_index: u64,
    address_index: u64,
    label: String,
) {
    wallet
        .label_address(account_index, address_index, label)
        .await
        .unwrap()
}

pub async fn label_address_error_invalid_account_index(
    wallet: &WalletClient,
    account_index: u64,
    address_index: u64,
) {
    let label_err = wallet
        .label_address(account_index, address_index, "".to_string())
        .await
        .unwrap_err();
    assert_eq!(
        label_err.to_string(),
        "Server error: account index is out of bound"
    );
}

pub async fn label_address_error_invalid_address_index(
    wallet: &WalletClient,
    account_index: u64,
    address_index: u64,
) {
    let label_err = wallet
        .label_address(account_index, address_index, "".to_string())
        .await
        .unwrap_err();
    assert_eq!(
        label_err.to_string(),
        "Server error: address index is out of bound"
    );
}

pub async fn get_accounts(
    wallet: &WalletClient,
    tag: Option<String>,
    expected_accounts_data: GetAccountsData,
) {
    let accounts_data = wallet.get_accounts(tag).await.unwrap();
    assert_eq!(accounts_data, expected_accounts_data);
}

pub async fn get_accounts_error_unregistered_tag(wallet: &WalletClient, tag: String) {
    let accounts_data_err = wallet.get_accounts(Some(tag.clone())).await.unwrap_err();
    assert_eq!(
        accounts_data_err.to_string(),
        format!("Server error: Tag {tag} is unregistered.")
    );
}