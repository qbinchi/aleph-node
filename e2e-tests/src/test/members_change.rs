use crate::{accounts::accounts_from_seeds, Config};
use aleph_client::{
    create_connection, get_current_session, wait_for_finalized_block, wait_for_session, Connection,
    Header, KeyPair,
};
use primitives::AuthorityId;
use sp_core::Pair;
use substrate_api_client::AccountId;

const MINIMAL_TEST_SESSION_START: u32 = 7;
const ELECTION_STARTS: u32 = 4;

fn get_reserved_members<T: Clone>(accounts: &Vec<T>) -> Vec<T> {
    accounts[..2].to_vec()
}

fn get_non_reserved_members_for_session<T: Clone>(accounts: &Vec<T>, session: u32) -> Vec<T> {
    // Test assumption
    const FREE_SITS: u32 = 2;

    let mut non_reserved = vec![];

    let x = accounts[2..].to_vec();
    let x_len = x.len();

    for i in (FREE_SITS * session)..(FREE_SITS * (session + 1)) {
        non_reserved.push(x[i as usize % x_len].clone());
    }

    non_reserved
}

fn get_authorities_for_session(connection: &Connection, session: u32) -> Vec<AccountId> {
    const SESSION_PERIOD: u32 = 30;
    let first_block = SESSION_PERIOD * session;

    let block = connection
        .get_block_hash(Some(first_block))
        .expect("Api call should succeed")
        .expect("Session already started so the first block should be present");

    connection
        .get_storage_value("Session", "Validators", Some(block))
        .expect("Api call should succeed")
        .expect("Authorities should always be present")
}

pub fn change_members(cfg: &Config) -> anyhow::Result<()> {
    let node = &cfg.node;
    let accounts = accounts_from_seeds(&None);
    let sender = accounts.first().expect("Using default accounts").to_owned();
    let connection = create_connection(node).set_signer(sender);

    let mut current_session = get_current_session(&connection);
    if current_session < MINIMAL_TEST_SESSION_START {
        wait_for_session(&connection, MINIMAL_TEST_SESSION_START)?;
        current_session = MINIMAL_TEST_SESSION_START;
    }

    let reserved_members: Vec<_> = get_reserved_members(&accounts)
        .iter()
        .map(|pair| AccountId::from(pair.public()))
        .collect();

    let accounts: Vec<_> = accounts
        .iter()
        .map(|pair| AccountId::from(pair.public()))
        .collect();

    for session in ELECTION_STARTS..current_session {
        let elected = get_authorities_for_session(&connection, session);
        let non_reserved = get_non_reserved_members_for_session(&accounts, session);

        let reserved_included = reserved_members
            .clone()
            .iter()
            .all(|reserved| elected.contains(reserved));

        let non_reserved_include = non_reserved
            .iter()
            .all(|non_reserved| elected.contains(non_reserved));

        let only_expected_members = elected
            .iter()
            .all(|elected| reserved_members.contains(elected) || non_reserved.contains(elected));

        assert!(
            reserved_included,
            "Reserved nodes should always be present, session #{}",
            session
        );
        assert!(
            non_reserved_include,
            "Missing non reserved node, session #{}",
            session
        );
        assert!(
            only_expected_members,
            "Only expected members should be present, session #{}",
            session
        );
    }

    let block_number = connection
        .get_header::<Header>(None)
        .expect("Could not fetch header")
        .expect("Block exists; qed")
        .number;
    wait_for_finalized_block(&connection, block_number)?;

    Ok(())
}
