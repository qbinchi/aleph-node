use log::{info, warn};
use sp_core::sr25519;
use sp_runtime::{generic::Header as GenericHeader, traits::BlakeTwo256};
use std::{thread::sleep, time::Duration};
use substrate_api_client::{rpc::ws_client::WsRpcClient, Api, RpcClient, XtStatus};

mod rpc;
mod session;
mod waiting;

pub use rpc::rotate_keys;
pub use session::{
    change_members, get_current as get_current_session, set_keys, wait_for as wait_for_session,
    Keys as SessionKeys,
};
pub use waiting::wait_for_event;

pub trait FromStr: Sized {
    type Err;

    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

impl FromStr for WsRpcClient {
    type Err = ();

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        Ok(WsRpcClient::new(url))
    }
}

pub type BlockNumber = u32;
pub type Header = GenericHeader<BlockNumber, BlakeTwo256>;
pub type KeyPair = sr25519::Pair;
pub type Connection = Api<KeyPair, WsRpcClient>;

pub fn create_connection(address: &str) -> Connection {
    create_custom_connection(address).expect("connection should be created")
}

pub fn create_custom_connection<Client: FromStr + RpcClient>(
    address: &str,
) -> Result<Api<sr25519::Pair, Client>, <Client as FromStr>::Err> {
    let client = Client::from_str(&format!("ws://{}", address))?;
    match Api::<sr25519::Pair, _>::new(client) {
        Ok(api) => Ok(api),
        Err(why) => {
            warn!(
                "[+] Can't create_connection because {:?}, will try again in 1s",
                why
            );
            sleep(Duration::from_millis(1000));
            create_custom_connection(address)
        }
    }
}

pub fn send_xt(connection: &Connection, xt: String, xt_name: &'static str, tx_status: XtStatus) {
    let block_hash = connection
        .send_extrinsic(xt, tx_status)
        .expect("Could not send extrinsic")
        .expect("Could not get tx hash");
    let block_number = connection
        .get_header::<Header>(Some(block_hash))
        .expect("Could not fetch header")
        .expect("Block exists; qed")
        .number;
    info!(
        "Transaction {} was included in block {}.",
        xt_name, block_number
    );
}