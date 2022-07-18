use monero::{KeyPair, PrivateKey};
use std::str::FromStr;

pub mod daemon_rpc_test;
pub mod regtest_test;
pub mod wallet_test;

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
