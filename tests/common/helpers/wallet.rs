use std::{collections::HashMap, num::NonZeroU64, str::FromStr};

use monero::{Address, Amount, Hash, PrivateKey};
use monero_rpc::{
    AddressData, BalanceData, GenerateFromKeysArgs, GetAccountsData, GotTransfer, PrivateKeyType,
    TransferData, TransferOptions, TransferPriority, WalletClient, WalletCreation,
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

pub async fn get_height(wallet: &WalletClient, expected_height: u64) {
    let height = wallet.get_height().await.unwrap();
    assert_eq!(height.get(), expected_height);
}

pub async fn refresh(
    wallet: &WalletClient,
    start_height: Option<u64>,
    expected_received_money: bool,
) {
    let res = wallet.refresh(start_height).await.unwrap();
    assert_eq!(res.received_money, expected_received_money);
}

pub async fn refresh_error(wallet: &WalletClient) {
    let err = wallet.refresh(None).await.unwrap_err();
    assert_eq!(err.to_string(), "Server error: No wallet file");
}

pub async fn query_key(wallet: &WalletClient, key_type: PrivateKeyType, expected_key: PrivateKey) {
    let key = wallet.query_key(key_type).await.unwrap();
    assert_eq!(key, expected_key);
}

pub async fn query_key_error_query_spend_key_for_view_only_wallet(wallet: &WalletClient) {
    let key_err = wallet.query_key(PrivateKeyType::Spend).await.unwrap_err();
    assert_eq!(
        key_err.to_string(),
        "Server error: The wallet is watch-only. Cannot retrieve spend key."
    );
}

pub async fn get_balance(
    wallet: &WalletClient,
    account_index: u64,
    address_indices: Option<Vec<u64>>,
    expected_balance_data: BalanceData,
) -> BalanceData {
    let balance_data = wallet
        .get_balance(account_index, address_indices)
        .await
        .unwrap();
    assert_eq!(balance_data, expected_balance_data);
    balance_data
}

pub async fn transfer(
    wallet: &WalletClient,
    destinations: HashMap<Address, Amount>,
    options: TransferOptions,
    priority: TransferPriority,
) -> TransferData {
    let t = wallet
        .transfer(destinations.clone(), priority, options)
        .await
        .unwrap();
    let dest_amount: u64 = destinations.into_values().map(|a| a.as_pico()).sum();
    assert_eq!(t.amount, dest_amount);
    t
}

pub async fn transfer_error_invalid_balance(
    wallet: &WalletClient,
    destinations: HashMap<Address, Amount>,
    options: TransferOptions,
) {
    let err = wallet
        .transfer(destinations, TransferPriority::Default, options)
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "Server error: not enough money");
}

pub async fn transfer_error_invalid_address(
    wallet: &WalletClient,
    destinations: HashMap<Address, Amount>,
    options: TransferOptions,
    wrong_address: Address,
) {
    let err = wallet
        .transfer(destinations, TransferPriority::Default, options)
        .await
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        format!(
            "Server error: WALLET_RPC_ERROR_CODE_WRONG_ADDRESS: {}",
            wrong_address
        )
    );
}

pub async fn transfer_error_payment_id_obsolete(
    wallet: &WalletClient,
    destinations: HashMap<Address, Amount>,
    options: TransferOptions,
) {
    let err = wallet
        .transfer(destinations, TransferPriority::Default, options)
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "Server error: Standalone payment IDs are obsolete. Use subaddresses or integrated addresses instead");
}

pub async fn relay_tx(wallet: &WalletClient, tx_metadata_hex: String, expected_tx_hash: String) {
    let res = wallet.relay_tx(tx_metadata_hex).await.unwrap();
    assert_eq!(res.to_string(), expected_tx_hash);
}

pub async fn relay_tx_error_invalid_hex(wallet: &WalletClient, tx_metadata_hex: String) {
    let err = wallet.relay_tx(tx_metadata_hex).await.unwrap_err();
    assert_eq!(err.to_string(), "Server error: Failed to parse hex.");
}

pub async fn relay_tx_error_invalid_tx_metadata(wallet: &WalletClient, tx_metadata_hex: String) {
    let err = wallet.relay_tx(tx_metadata_hex).await.unwrap_err();
    assert_eq!(
        err.to_string(),
        "Server error: Failed to parse tx metadata."
    );
}

pub async fn get_transfer(
    wallet: &WalletClient,
    txid: Hash,
    account_index: Option<u64>,
    mut expected_got_transfer: Option<GotTransfer>,
) {
    let transfer = wallet.get_transfer(txid, account_index).await.unwrap();

    if let Some(ref mut t) = expected_got_transfer {
        t.timestamp = transfer.as_ref().unwrap().timestamp;
    }

    assert_eq!(transfer, expected_got_transfer);
}

pub async fn get_transfer_error_invalid_txid(wallet: &WalletClient, txid: Hash) {
    let transfer_err = wallet.get_transfer(txid, None).await.unwrap();
    assert_eq!(transfer_err, None);
}

pub async fn get_transfer_error_invalid_account_index(
    wallet: &WalletClient,
    txid: Hash,
    account_index: Option<u64>,
) {
    let transfer_err = wallet.get_transfer(txid, account_index).await.unwrap_err();
    assert_eq!(
        transfer_err.to_string(),
        "Server error: Account index is out of bound"
    );
}

pub async fn check_tx_key(
    wallet: &WalletClient,
    txid: Hash,
    tx_key: Vec<u8>,
    address: Address,
    expected_res: (u64, bool, u64),
) {
    let res = wallet.check_tx_key(txid, tx_key, address).await.unwrap();
    assert_eq!(res, expected_res);
}

pub async fn check_tx_key_error_invalid_txid(
    wallet: &WalletClient,
    txid: Hash,
    tx_key: Vec<u8>,
    address: Address,
) {
    let err = wallet
        .check_tx_key(txid, tx_key, address)
        .await
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Server error: Failed to get transaction from daemon"
    );
}

pub async fn check_tx_key_error_invalid_tx_key(
    wallet: &WalletClient,
    txid: Hash,
    tx_key: Vec<u8>,
    address: Address,
) {
    let err = wallet
        .check_tx_key(txid, tx_key, address)
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "Server error: Tx key has invalid format");
}

pub async fn check_tx_key_error_invalid_address(
    wallet: &WalletClient,
    txid: Hash,
    tx_key: Vec<u8>,
    address: Address,
) {
    let err = wallet
        .check_tx_key(txid, tx_key, address)
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "Server error: Invalid address");
}

pub async fn export_key_images() {
    assert!(false);
}

pub async fn export_key_images_error(wallet: &WalletClient) {
    assert!(false);
}

pub async fn import_key_images() {
    assert!(false);
}

pub async fn import_key_images_error_empty_vec() {
    assert!(false);
}

pub async fn incoming_transfers() {
    assert!(false);
}

pub async fn incoming_transfers_error_no_transfer_for_type() {
    assert!(false);
}

pub async fn incoming_transfers_error_invalid_account_index() {
    assert!(false);
}

pub async fn incoming_transfers_error_invalid_subaddr_indices() {
    assert!(false);
}

pub async fn sign_transfer() {
    assert!(false);
}

pub async fn sign_transfer_error_invalid_hex() {
    assert!(false);
}

pub async fn sign_transfer_error_invalid_unsigned_txset() {
    assert!(false);
}

pub async fn submit_transfer() {
    assert!(false);
}

pub async fn submit_transfer_error_invalid_hex() {
    assert!(false);
}

pub async fn submit_transfer_error_invalid_unsigned_txset() {
    assert!(false);
}
