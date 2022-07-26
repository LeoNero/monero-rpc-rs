use std::{collections::HashMap, num::NonZeroU64};

use chrono::{DateTime, NaiveDateTime, Utc};
use hex::ToHex;
use monero::{
    consensus::Decodable,
    cryptonote::subaddress::{self, Index},
    util::address::PaymentId,
    Address, Amount, Hash, KeyPair, Network, TxIn, ViewPair,
};
use monero_rpc::{
    BalanceData, GetTransfersCategory, GotTransfer, HashString, IncomingTransfer,
    IncomingTransfers, KeyImageImportResponse, Payment, PrivateKeyType, SubaddressBalanceData,
    SubaddressIndex, Transaction, TransactionsResponse, TransferHeight, TransferOptions,
    TransferPriority, TransferType,
};

use crate::common::helpers;

pub async fn test() {
    let (regtest, daemon_rpc, wallet) = helpers::setup_monero();

    // it is important for this wallet to be non-deterministic instead of being generated from some
    // keypair from some `helpers::get_keypair_`, so that any transfer this wallet receives won't
    // be there when tests run again.
    let wallet_1_full = helpers::wallet::create_wallet_with_empty_password(&wallet).await;
    let wallet_1_key_pair = KeyPair {
        view: wallet.query_key(PrivateKeyType::View).await.unwrap(),
        spend: wallet.query_key(PrivateKeyType::Spend).await.unwrap(),
    };
    let wallet_1_address = Address::from_keypair(Network::Mainnet, &wallet_1_key_pair);
    let (wallet_1_view_only, wallet_1_view_only_creation_data) =
        helpers::wallet::generate_from_keys(
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

    // also important to be non-deterministic, for same reasons as wallet_1
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
    let mut transfer_1_destination: HashMap<Address, Amount> = HashMap::new();
    transfer_1_destination.insert(wallet_1_address, Amount::from_xmr(5.0).unwrap());

    let mut transfer_options = TransferOptions {
        account_index: None,
        subaddr_indices: None,
        mixin: None,
        ring_size: None,
        unlock_time: None,
        payment_id: None,
        do_not_relay: None,
    };

    transfer_1_destination.insert(wallet_2_subaddress_1, Amount::from_xmr(40.0).unwrap());
    helpers::wallet::transfer_error_invalid_balance(
        &wallet,
        transfer_1_destination.clone(),
        transfer_options.clone(),
    )
    .await;

    // change to an amount that fits in the balance...
    transfer_1_destination
        .entry(wallet_2_subaddress_1)
        .and_modify(|e| *e = Amount::from_xmr(10.0).unwrap());

    // ... but add an invalid address ...
    let wallet_3_testnet_address =
        Address::from_keypair(Network::Testnet, &helpers::get_keypair_1());
    transfer_1_destination.insert(wallet_3_testnet_address, Amount::from_xmr(40.0).unwrap());
    helpers::wallet::transfer_error_invalid_address(
        &wallet,
        transfer_1_destination.clone(),
        transfer_options.clone(),
        wallet_3_testnet_address,
    )
    .await;

    // ... remove the invalid address but add a 'wrong' account_index...
    transfer_1_destination
        .remove(&wallet_3_testnet_address)
        .unwrap();
    transfer_options.account_index = Some(10);
    helpers::wallet::transfer_error_invalid_balance(
        &wallet,
        transfer_1_destination.clone(),
        transfer_options.clone(),
    )
    .await;

    // ... go back to correct account_index, but add 'invalid' subaddr_index...
    transfer_options.account_index = None;
    transfer_options.subaddr_indices = Some(vec![10]);
    helpers::wallet::transfer_error_invalid_balance(
        &wallet,
        transfer_1_destination.clone(),
        transfer_options.clone(),
    )
    .await;

    // ... restore subaddr_index and send transaction
    transfer_options.subaddr_indices = None;
    let transfer_1_data = helpers::wallet::transfer(
        &wallet,
        transfer_1_destination.clone(),
        transfer_options,
        TransferPriority::Default,
    )
    .await;
    helpers::wallet::refresh(&wallet, None, false).await;

    // ... try to relay it again...
    helpers::wallet::relay_tx(
        &wallet,
        transfer_1_data.tx_metadata.to_string(),
        transfer_1_data.tx_hash.0.to_string(),
    )
    .await;

    // relay_tx errors
    helpers::wallet::relay_tx_error_invalid_hex(&wallet, "01234".to_string()).await;
    let mut wrong_tx_metadata = transfer_1_data.tx_metadata.to_string();
    wrong_tx_metadata.replace_range(100..105, "6");
    helpers::wallet::relay_tx_error_invalid_tx_metadata(&wallet, wrong_tx_metadata).await;

    // obsolete payment ids
    let transfer_options = TransferOptions {
        account_index: Some(0),
        subaddr_indices: Some(vec![1]),
        mixin: Some(1000),
        ring_size: Some(8),
        unlock_time: Some(20),
        payment_id: Some(PaymentId::zero()),
        do_not_relay: Some(true),
    };
    helpers::wallet::transfer_error_payment_id_obsolete(
        &wallet,
        transfer_1_destination.clone(),
        transfer_options,
    )
    .await;

    // test daemon_rpc
    helpers::daemon_rpc::get_transactions_as_hex_not_pruned(
        &daemon_rpc,
        vec![transfer_1_data.tx_hash.0],
        TransactionsResponse {
            credits: 0,
            top_hash: "".to_string(),
            status: "OK".to_string(),
            missed_tx: None,
            txs: Some(vec![Transaction {
                as_hex: transfer_1_data.tx_blob.0.encode_hex(),
                as_json: Some("".to_string()),
                double_spend_seen: false,
                in_pool: true,
                tx_hash: transfer_1_data.tx_hash.clone(),
                block_height: None,
                block_timestamp: None,
                output_indices: None,
            }]),
            txs_as_hex: Some(vec![transfer_1_data.tx_blob.0.encode_hex()]),
            txs_as_json: None,
            untrusted: false,
        },
    )
    .await;
    helpers::daemon_rpc::get_transactions_as_hex_pruned(
        &daemon_rpc,
        vec![transfer_1_data.tx_hash.0],
        TransactionsResponse {
            credits: 0,
            top_hash: "".to_string(),
            status: "OK".to_string(),
            missed_tx: None,
            txs: Some(vec![Transaction {
                as_hex: "".to_string(),
                as_json: Some("".to_string()),
                double_spend_seen: false,
                in_pool: true,
                tx_hash: transfer_1_data.tx_hash.clone(),
                block_height: None,
                block_timestamp: None,
                output_indices: None,
            }]),
            txs_as_hex: Some(vec!["".to_string()]),
            txs_as_json: None,
            untrusted: false,
        },
    )
    .await;
    // the functions below only test if the _json fields are not none
    helpers::daemon_rpc::get_transactions_as_json_not_pruned(
        &daemon_rpc,
        vec![transfer_1_data.tx_hash.0],
    )
    .await;
    helpers::daemon_rpc::get_transactions_as_json_pruned(
        &daemon_rpc,
        vec![transfer_1_data.tx_hash.0],
    )
    .await;

    // get_transfer
    let expected_got_transfer = Some(GotTransfer {
        address: wallet_2_address,
        amount: 15000000000000,
        confirmations: None,
        double_spend_seen: false,
        fee: transfer_1_data.fee,
        height: TransferHeight::InPool,
        note: "".to_string(),
        payment_id: HashString(PaymentId::zero()),
        subaddr_index: SubaddressIndex { major: 0, minor: 0 },
        suggested_confirmations_threshold: 1,
        // this is any date, since it will not be tested against anything
        timestamp: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
        txid: HashString(transfer_1_data.tx_hash.0.as_ref().to_vec()),
        transfer_type: GetTransfersCategory::Pending,
        unlock_time: 0,
    });
    helpers::wallet::get_transfer(
        &wallet,
        transfer_1_data.tx_hash.0,
        Some(0),
        expected_got_transfer,
    )
    .await;
    helpers::wallet::get_transfer_error_invalid_txid(&wallet, Hash::zero()).await;
    helpers::wallet::get_transfer_error_invalid_account_index(
        &wallet,
        transfer_1_data.tx_hash.0,
        Some(1000),
    )
    .await;

    // check_tx_key
    helpers::wallet::check_tx_key(
        &wallet,
        transfer_1_data.tx_hash.0,
        transfer_1_data.tx_key.0.clone(),
        wallet_1_address,
        (0, true, transfer_1_destination[&wallet_1_address].as_pico()),
    )
    .await;
    helpers::wallet::check_tx_key(
        &wallet,
        transfer_1_data.tx_hash.0,
        transfer_1_data.tx_key.0.clone(),
        wallet_2_address,
        // wallet_2 has just one output of value expected_balance;
        // it uses such outout in the transaction
        // thus, the last value of the tuple is the change
        (
            0,
            true,
            expected_balance - transfer_1_data.amount - transfer_1_data.fee,
        ),
    )
    .await;
    helpers::wallet::check_tx_key_error_invalid_txid(
        &wallet,
        Hash::zero(),
        transfer_1_data.tx_key.0.clone(),
        wallet_2_address,
    )
    .await;
    helpers::wallet::check_tx_key_error_invalid_tx_key(
        &wallet,
        transfer_1_data.tx_hash.0,
        vec![1, 2, 3, 4],
        wallet_2_address,
    )
    .await;
    helpers::wallet::check_tx_key_error_invalid_address(
        &wallet,
        transfer_1_data.tx_hash.0,
        transfer_1_data.tx_key.0.clone(),
        wallet_3_testnet_address,
    )
    .await;

    // export_key_images for wallet_2...
    // should be empty
    helpers::wallet::export_key_images_empty(&wallet).await;

    // ... and change to wallet_1_full, refresh and export_key_images of it (it has offset 1, and
    // returns empty vec)...
    helpers::wallet::close_wallet(&wallet).await;
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_full).await;
    helpers::wallet::refresh(&wallet, None, false).await;
    helpers::wallet::export_key_images_empty(&wallet).await;

    // ... now change to wallet_1_view_only, refresh, and export_key_images of it...
    helpers::wallet::close_wallet(&wallet).await;
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_view_only).await;
    helpers::wallet::refresh(&wallet, None, false).await;
    helpers::wallet::export_key_images_empty(&wallet).await;

    // ... change to wallet with no key images and test what is returned ...
    let temp_wallet = helpers::wallet::create_wallet_with_empty_password(&wallet).await;
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &temp_wallet).await;
    helpers::wallet::refresh(&wallet, None, false).await;
    helpers::wallet::export_key_images_empty(&wallet).await;

    // ... go back to wallet_2 and import_key_images of wallet_1_full, which is empty
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_2).await;
    helpers::wallet::import_key_images_empty_vec(&wallet).await;

    // change to wallet_1_view_only, and test incoming_transfers...
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_view_only).await;
    helpers::wallet::refresh(&wallet, None, false).await;
    let expected_incoming_transfers = IncomingTransfers { transfers: None };
    helpers::wallet::incoming_transfers(
        &wallet,
        TransferType::All,
        Some(0),
        Some(vec![0, 1, 2]),
        expected_incoming_transfers.clone(),
    )
    .await;

    // ...change to wallet_1_full, and test incoming_transfers
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_full).await;
    helpers::wallet::refresh(&wallet, None, false).await;
    let expected_incoming_transfers = IncomingTransfers { transfers: None };
    helpers::wallet::incoming_transfers(
        &wallet,
        TransferType::All,
        Some(0),
        Some(vec![0, 1, 2]),
        expected_incoming_transfers.clone(),
    )
    .await;

    // incoming_transfers variations
    helpers::wallet::incoming_transfers(
        &wallet,
        TransferType::Unavailable,
        None,
        None,
        expected_incoming_transfers.clone(),
    )
    .await;
    helpers::wallet::incoming_transfers(
        &wallet,
        TransferType::All,
        Some(100),
        None,
        expected_incoming_transfers.clone(),
    )
    .await;
    helpers::wallet::incoming_transfers(
        &wallet,
        TransferType::All,
        Some(0),
        Some(vec![1000]),
        expected_incoming_transfers.clone(),
    )
    .await;

    // mine some blocks to settle transfers...
    let height_before_settling_transfer_1 = wallet.get_height().await.unwrap().get();
    helpers::regtest::generate_blocks(&regtest, 10, wallet_3_address).await;
    helpers::wallet::refresh(&wallet, None, true).await;

    // ... and test export_key_images, import_key_images, and incoming_transfers for wallet_1_full again ...

    // ... starting with export_key_images... note: export_key_images has offset 1 for wallet_1_full and returns empty vec; it has offset
    // 0 for wallet_1_view_only and does not return empty.
    helpers::wallet::export_key_images_empty(&wallet).await;
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_view_only).await;
    wallet.refresh(Some(0)).await.unwrap();
    // Below is commented because it sometimes returns `true`, and sometimes `false`
    // helpers::wallet::refresh(&wallet, Some(0), true).await;
    let wallet_1_view_only_key_images = helpers::wallet::export_key_images(&wallet).await;
    // change to wallet_2 to import key images from wallet_1_view_only
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_2).await;
    helpers::wallet::import_key_images(
        &wallet,
        wallet_1_view_only_key_images,
        KeyImageImportResponse {
            height: 0,
            spent: 0,
            unspent: 0,
        },
    )
    .await;

    // ...  and then incoming_transfers, for both wallet_1_full and wallet_1_view_only
    let expected_incoming_transfers = IncomingTransfers {
        transfers: Some(vec![IncomingTransfer {
            global_index: 0, // this is any number, since we will not test against it
            key_image: None, // this is different from the key_image in the Inputs for transfer_1_data, so we set it to None and do not test it
            tx_size: None,   // any value, since we will not test againt it
            amount: transfer_1_destination[&wallet_1_address].as_pico(),
            spent: false,
            subaddr_index: SubaddressIndex { major: 0, minor: 0 },
            tx_hash: transfer_1_data.tx_hash.clone(),
        }]),
    };
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_view_only).await;
    helpers::wallet::incoming_transfers(
        &wallet,
        TransferType::All,
        Some(0),
        Some(vec![0, 1, 2]),
        expected_incoming_transfers.clone(),
    )
    .await;
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_full).await;
    helpers::wallet::incoming_transfers(
        &wallet,
        TransferType::All,
        Some(0),
        Some(vec![0, 1, 2]),
        expected_incoming_transfers,
    )
    .await;

    // wallet_1_view_only is read-only, so `transfer` will create an unsigned_txset, which is then used in `sign_transfer`...
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_view_only).await;
    let mut transfer_2_destination: HashMap<Address, Amount> = HashMap::new();
    transfer_2_destination.insert(wallet_2_address, Amount::from_xmr(0.00001).unwrap());
    let transfer_2_data_unsigned = helpers::wallet::transfer(
        &wallet,
        transfer_2_destination,
        TransferOptions {
            account_index: None,
            subaddr_indices: None,
            mixin: None,
            ring_size: None,
            unlock_time: None,
            payment_id: None,
            do_not_relay: None,
        },
        TransferPriority::Unimportant,
    )
    .await;
    // ... we then go to `wallet_1_full`, so that we can sign the transaction
    helpers::wallet::open_wallet_with_no_or_empty_password(&wallet, &wallet_1_full).await;
    let transfer_2_data_signed =
        helpers::wallet::sign_transfer(&wallet, transfer_2_data_unsigned.unsigned_txset.0.clone())
            .await;
    helpers::wallet::sign_transfer_error_cannot_load(&wallet, vec![0, 1, 2, 3]).await;
    let mut invalid_unsigned_txset = transfer_2_data_unsigned.unsigned_txset.0;
    for e in invalid_unsigned_txset.iter_mut().take(25 + 1).skip(20) {
        *e = 5;
    }
    helpers::wallet::sign_transfer_error_cannot_load(&wallet, invalid_unsigned_txset.clone()).await;

    // ... and submit transfer after that
    helpers::wallet::submit_transfer(&wallet, transfer_2_data_signed.signed_txset.clone()).await;
    helpers::wallet::submit_transfer_error_parse(&wallet, vec![0, 1, 2, 3]).await;
    let mut invalid_signed_txset = transfer_2_data_signed.signed_txset;
    for e in invalid_signed_txset.iter_mut().take(25 + 1).skip(20) {
        *e = 5;
    }
    helpers::wallet::submit_transfer_error_parse(&wallet, invalid_signed_txset).await;

    // get_payments and get_bulk_payments
    let expected_payment_ids = vec![Payment {
        address: wallet_1_address,
        payment_id: HashString(PaymentId::zero()),
        tx_hash: transfer_1_data.tx_hash,
        amount: transfer_1_destination[&wallet_1_address].as_pico(),
        unlock_time: 0,
        subaddr_index: SubaddressIndex { major: 0, minor: 0 },
        block_height: height_before_settling_transfer_1,
    }];
    helpers::wallet::get_payments(&wallet, PaymentId::zero(), expected_payment_ids.clone()).await;
    helpers::wallet::get_payments(&wallet, PaymentId::repeat_byte(10), vec![]).await;
    helpers::wallet::get_bulk_payments(
        &wallet,
        vec![PaymentId::zero(), PaymentId::repeat_byte(10)],
        0,
        expected_payment_ids,
    )
    .await;
    helpers::wallet::get_bulk_payments(
        &wallet,
        vec![PaymentId::zero(), PaymentId::repeat_byte(10)],
        u64::MAX - 100000,
        vec![],
    )
    .await;
}

/*
* TODO
async fn functional_wallet_test() {
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
