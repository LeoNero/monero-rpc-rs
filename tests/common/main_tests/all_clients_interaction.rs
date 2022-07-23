use std::collections::HashMap;

use monero::{
    cryptonote::subaddress::{self, Index},
    Address, Amount, KeyPair, Network, ViewPair,
};
use monero_rpc::{
    BalanceData, PrivateKeyType, SubaddressBalanceData, TransferOptions, TransferPriority,
};

use crate::common::helpers;

pub async fn test() {
    let (regtest, daemon_rpc, wallet) = helpers::setup_monero();

    let wallet_1_key_pair = helpers::get_keypair_3();
    let wallet_1_address = Address::from_keypair(Network::Mainnet, &wallet_1_key_pair);
    let (wallet_1, wallet_1_creation_data) = helpers::wallet::generate_from_keys(
        &wallet,
        monero_rpc::GenerateFromKeysArgs {
            restore_height: Some(0),
            filename: "".to_string(), // empty, so random name is assigned
            address: wallet_1_address,
            spendkey: None,
            viewkey: wallet_1_key_pair.view,
            password: "".to_string(),
            autosave_current: None,
        },
    )
    .await;

    helpers::wallet::query_key(&wallet, PrivateKeyType::View, wallet_1_key_pair.view).await;
    helpers::wallet::query_key_error_query_spend_key_for_view_only_wallet(&wallet).await;

    let wallet_2 = helpers::wallet::create_wallet_with_empty_password(&wallet).await;
    // when created, `height` returned by wallet.get_height is a bit inconsistent (sometimes
    // returns 1, sometimes returns the correct result), so we ignore it
    // helpers::wallet::get_height(&wallet, 1).await;

    // note the order of the following two `refresh` is important
    // no error for invalid height
    helpers::wallet::refresh(&wallet, Some(u64::MAX), false).await;
    // we refresh the wallet to catch up with the network, and make sure get_height returns the
    // correct result
    helpers::wallet::refresh(&wallet, None, false).await;

    let block_count = regtest.get_block_count().await.unwrap().get();
    let expected_wallet_height = block_count;
    // **Note**: the height returned by a fully-synced wallet is equal to the number of blocks.
    // If `wallet_height` is the response of `get_height`, then daemon's `get_block_header_by_height(wallet_height)`
    // returns an error
    helpers::wallet::get_height(&wallet, expected_wallet_height).await;
    let current_height = block_count - 1;
    helpers::regtest::get_block_header_at_height_error(
        &regtest,
        expected_wallet_height,
        current_height,
    )
    .await;

    // close and refresh wallet; then open it again
    helpers::wallet::close_wallet(&wallet).await;
    helpers::wallet::refresh_error(&wallet).await;
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_2).await;

    // query keys of `wallet_2` and get its address
    let wallet_2_key_pair = KeyPair {
        view: wallet.query_key(PrivateKeyType::View).await.unwrap(),
        spend: wallet.query_key(PrivateKeyType::Spend).await.unwrap(),
    };
    let wallet_2_address = Address::from_keypair(Network::Mainnet, &wallet_2_key_pair);

    // create a subaddress for `wallet_2 and mine a block on the main address and on the
    // subaddress; check the balance at the end
    let wallet_2_subaddress_1 = subaddress::get_subaddress(
        &ViewPair::from(&wallet_2_key_pair),
        Index { major: 0, minor: 1 },
        Some(Network::Mainnet),
    );
    let wallet_2_subaddress_1_label = "faaaarcaster".to_string();
    helpers::wallet::create_address(
        &wallet,
        0,
        Some(wallet_2_subaddress_1_label.clone()),
        (wallet_2_subaddress_1, 1),
    )
    .await;

    let expected_balance = regtest
        .get_block_template(wallet_2_address, 0)
        .await
        .unwrap()
        .expected_reward;
    helpers::regtest::generate_blocks(&regtest, 1, wallet_2_address).await;
    helpers::regtest::generate_blocks_error_subaddress_not_supported(
        &regtest,
        wallet_2_subaddress_1,
    )
    .await;

    helpers::wallet::refresh(&wallet, Some(0), true).await;

    let expected_balance_data_for_wallet_2 = BalanceData {
        balance: expected_balance,
        unlocked_balance: 0,
        multisig_import_needed: false,
        per_subaddress: vec![SubaddressBalanceData {
            address: wallet_2_address,
            address_index: 0,
            balance: expected_balance,
            label: "Primary account".to_string(),
            num_unspent_outputs: 1,
            unlocked_balance: 0,
        }],
    };
    helpers::wallet::get_balance(&wallet, 0, None, expected_balance_data_for_wallet_2).await;
    let expected_balance_data_for_wallet_2_subaddress_1 = BalanceData {
        balance: expected_balance,
        unlocked_balance: 0,
        multisig_import_needed: false,
        per_subaddress: vec![SubaddressBalanceData {
            address: wallet_2_subaddress_1,
            address_index: 1,
            balance: 0,
            label: wallet_2_subaddress_1_label,
            num_unspent_outputs: 0,
            unlocked_balance: 0,
        }],
    };
    helpers::wallet::get_balance(
        &wallet,
        0,
        Some(vec![1]),
        expected_balance_data_for_wallet_2_subaddress_1,
    )
    .await;

    // No error for weird account and address index
    let wallet_2_subaddress_12345678 = subaddress::get_subaddress(
        &ViewPair::from(&wallet_2_key_pair),
        Index {
            major: 0,
            minor: 12345678,
        },
        Some(Network::Mainnet),
    );
    let expected_balance_data_for_wallet_2_subaddress_12345678 = BalanceData {
        balance: expected_balance,
        unlocked_balance: 0,
        multisig_import_needed: false,
        per_subaddress: vec![SubaddressBalanceData {
            address: wallet_2_subaddress_12345678,
            address_index: 12345678,
            balance: 0,
            label: "".to_string(),
            num_unspent_outputs: 0,
            unlocked_balance: 0,
        }],
    };
    helpers::wallet::get_balance(
        &wallet,
        0,
        Some(vec![12345678]),
        expected_balance_data_for_wallet_2_subaddress_12345678,
    )
    .await;

    let expected_balance_data_for_wallet_2_invalid_account = BalanceData {
        balance: 0,
        unlocked_balance: 0,
        multisig_import_needed: false,
        per_subaddress: vec![],
    };
    helpers::wallet::get_balance(
        &wallet,
        10000000, // u64::MAX returns error...
        None,
        expected_balance_data_for_wallet_2_invalid_account,
    )
    .await;

    // mine 59 blocks to another address, so that wallet_2 can have unlocked balance
    let wallet_3_address = Address::from_keypair(Network::Mainnet, &helpers::get_keypair_1());
    helpers::regtest::generate_blocks(&regtest, 59, wallet_3_address).await;
    helpers::wallet::refresh(&wallet, None, false).await;
    let expected_balance_data_for_wallet_2 = BalanceData {
        balance: expected_balance,
        unlocked_balance: expected_balance,
        multisig_import_needed: false,
        per_subaddress: vec![SubaddressBalanceData {
            address: wallet_2_address,
            address_index: 0,
            balance: expected_balance,
            label: "Primary account".to_string(),
            num_unspent_outputs: 1,
            unlocked_balance: expected_balance,
        }],
    };
    helpers::wallet::get_balance(&wallet, 0, None, expected_balance_data_for_wallet_2).await;

    // transfers and transactions
    let mut destination: HashMap<Address, Amount> = HashMap::new();
    destination.insert(wallet_1_address, Amount::from_xmr(5.0).unwrap());

    let mut transfer_options = TransferOptions {
        account_index: None,
        subaddr_indices: None,
        mixin: None,
        ring_size: None,
        unlock_time: None,
        payment_id: None,
        do_not_relay: None,
    };

    destination.insert(wallet_2_subaddress_1, Amount::from_xmr(40.0).unwrap());
    helpers::wallet::transfer_error_invalid_balance(
        &wallet,
        destination.clone(),
        transfer_options.clone(),
    )
    .await;

    // change to an amount that fits in the balance...
    destination
        .entry(wallet_2_subaddress_1)
        .and_modify(|e| *e = Amount::from_xmr(10.0).unwrap());

    // ... but add an invalid address ...
    let wallet_3_testnet_address =
        Address::from_keypair(Network::Testnet, &helpers::get_keypair_1());
    destination.insert(wallet_3_testnet_address, Amount::from_xmr(40.0).unwrap());
    helpers::wallet::transfer_error_invalid_address(
        &wallet,
        destination.clone(),
        transfer_options.clone(),
        wallet_3_testnet_address,
    )
    .await;

    // ... remove the invalid address but add a 'wrong' account_index...
    destination.remove(&wallet_3_testnet_address).unwrap();
    transfer_options.account_index = Some(10);
    helpers::wallet::transfer_error_invalid_balance(
        &wallet,
        destination.clone(),
        transfer_options.clone(),
    )
    .await;

    // ... go back to correct account_index, but add 'invalid' subaddr_index...
    transfer_options.account_index = None;
    transfer_options.subaddr_indices = Some(vec![10]);
    helpers::wallet::transfer_error_invalid_balance(
        &wallet,
        destination.clone(),
        transfer_options.clone(),
    )
    .await;

    // ... restore subaddr_index and send transaction
    transfer_options.subaddr_indices = None;
    let transfer_1_data = helpers::wallet::transfer(
        &wallet,
        transfer_options,
        destination,
        TransferPriority::Default,
    )
    .await;
    helpers::wallet::refresh(&wallet, None, true).await;

    // ... try to relay it again...
    // TODO

    // ... check balances of wallet_1 and wallet_2_subaddress_1
    // TODO

    // create another transaction and do not relay, then relay
    // TODO
    // let mut transfer_options = TransferOptions {
    //     account_index: None,
    //     subaddr_indices: None,
    //     mixin: None,
    //     ring_size: None,
    //     unlock_time: None,
    //     payment_id: None,
    //     do_not_relay: None,
    // };
    // account_index != None, subaddr_indices != None, mixin != None, ring_size != None, payment_id =
    // Some(), do_not_relay = Some(true), then try try to relay again

    // TODO test daemon_rpc
}

/*
* TODO
async fn functional_wallet_test() {
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
        .open_wallet(view_wallet_name.clone(), None)
        .await
        .unwrap();
    wallet.export_key_images().await.unwrap();

    wallet.refresh(Some(0)).await.unwrap();

    wallet
        .incoming_transfers(monero_rpc::TransferType::All, Some(0), Some(vec![0, 1, 2]))
        .await
        .unwrap();

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
