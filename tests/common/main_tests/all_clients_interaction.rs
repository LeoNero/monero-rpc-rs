use crate::common::helpers;

pub async fn test() {
    let (regtest, daemon_rpc, wallet) = helpers::setup_monero();
}
