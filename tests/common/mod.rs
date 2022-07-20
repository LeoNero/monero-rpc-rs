use monero::{KeyPair, PrivateKey};
use monero_rpc::BlockHash;
use std::str::FromStr;

pub mod daemon_rpc_test;
pub mod regtest_test;
pub mod wallet_test;

pub const PWD_1: &str = "pwd_farcaster";

pub fn get_keypair_1() -> KeyPair {
    KeyPair {
        view: PrivateKey::from_str(
            "8ae33e57aee12fa4ad5b42a3ab093d9f3cb7f9be68b112a85f83275bcc5a190b",
        )
        .unwrap(),
        spend: PrivateKey::from_str(
            "eae5d41a112e14dcd549780a982bb3653c2f86ab1f4e6aa2b13c41f8b893ab04",
        )
        .unwrap(),
    }
}

pub fn get_keypair_2() -> KeyPair {
    KeyPair {
        view: PrivateKey::from_str(
            "21dbc3b71b900ac5af0d2e1cc3b279ad3b4a66633d1d8f6653b838f11bd14904",
        )
        .unwrap(),
        spend: PrivateKey::from_str(
            "90b7a822fbd3d06d04f5ad746300601a85c469ffc21b2fd7281cc43227537209",
        )
        .unwrap(),
    }
}

pub fn get_keypair_3() -> KeyPair {
    KeyPair {
        view: PrivateKey::from_str(
            "fb2a79386a470d1743cb867505bdf8045e2d44f012155018096740ea2fbca200",
        )
        .unwrap(),
        spend: PrivateKey::from_str(
            "a0967c33a4a1b6fbfdd0fba36f0593fa6627462470f9a58c09a532518415320a",
        )
        .unwrap(),
    }
}

pub fn get_keypair_4() -> KeyPair {
    KeyPair {
        view: PrivateKey::from_str(
            "feedd63638da7fbf433f892888a6d4c325ff7b127bfb01c021c3e7aa93f6c208",
        )
        .unwrap(),
        spend: PrivateKey::from_str(
            "a03b9c7a6ff4d6722334965811772d1320ce43f52d31ebe971c9f1dcbdbe090b",
        )
        .unwrap(),
    }
}

pub fn get_genesis_block_hash() -> BlockHash {
    BlockHash::from_str("418015bb9ae982a1975da7d79277c2705727a56894ba0fb246adaabb1f4632e3").unwrap()
}
