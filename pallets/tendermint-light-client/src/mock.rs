use super::*;
use crate::{self as tendermint_light_client, types::*, utils::timestamp_from_rfc3339};
use frame_support::{
    construct_runtime, parameter_types, sp_io,
    traits::{Everything, IsType},
    weights::RuntimeDbWeight,
};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    generic::BlockId,
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use tendermint::{
    block::{self, commit_sig},
    evidence, signature, validator, Version,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

pub const TRUSTED_BLOCK: &str = r#"{
    "signed_header": {
        "header": {
            "version": {
                "block": "11",
                "app": "0"
            },
            "chain_id": "test-chain",
            "height": "3",
            "time": "1970-01-01T00:00:03Z",
            "last_block_id": null,
            "last_commit_hash": null,
            "data_hash": null,
            "validators_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "next_validators_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "consensus_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "app_hash": "",
            "last_results_hash": null,
            "evidence_hash": null,
            "proposer_address": "6AE5C701F508EB5B63343858E068C5843F28105F"
        },
        "commit": {
            "height": "3",
            "round": 1,
            "block_id": {
                "hash": "AAB1B09D5FADAAE7CDF3451961A63F810DB73BF3214A7B74DBA36C52EDF1A793",
                "part_set_header": {
                    "total": 1,
                    "hash": "AAB1B09D5FADAAE7CDF3451961A63F810DB73BF3214A7B74DBA36C52EDF1A793"
                }
            },
            "signatures": [
                {
                    "block_id_flag": 2,
                    "validator_address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                    "timestamp": "1970-01-01T00:00:03Z",
                    "signature": "xn0eSsHYIsqUbmfAiJq1R0hqZbfuIjs5Na1c88EC1iPTuQAesKg9I7nXG4pk8d6U5fU4GysNLk5I4f7aoefOBA=="
                }
            ]
        }
    },
    "validator_set": {
        "total_voting_power": "50",
        "validators": [
            {
                "address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "GQEC/HB4sDBAVhHtUzyv4yct9ZGnudaP209QQBSTfSQ="
                },
                "voting_power": "50",
                "proposer_priority": null
            }
        ]
    },
    "next_validator_set": {
        "total_voting_power": "50",
        "validators": [
            {
                "address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "GQEC/HB4sDBAVhHtUzyv4yct9ZGnudaP209QQBSTfSQ="
                },
                "voting_power": "50",
                "proposer_priority": null
            }
        ]
    },
    "provider": "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE"
}"#;

pub const UNTRUSTED_BLOCK: &str = r#"{
    "signed_header": {
        "header": {
            "version": {
                "block": "11",
                "app": "0"
            },
            "chain_id": "test-chain",
            "height": "4",
            "time": "1970-01-01T00:00:04Z",
            "last_block_id": null,
            "last_commit_hash": null,
            "data_hash": null,
            "validators_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "next_validators_hash": "C8CFFADA9808F685C4111693E1ADFDDBBEE9B9493493BEF805419F143C5B0D0A",
            "consensus_hash": "75E6DD63C2DC2B58FE0ED82792EAB369C4308C7EC16B69446382CC4B41D46068",
            "app_hash": "",
            "last_results_hash": null,
            "evidence_hash": null,
            "proposer_address": "6AE5C701F508EB5B63343858E068C5843F28105F"
        },
        "commit": {
            "height": "4",
            "round": 1,
            "block_id": {
                "hash": "D0E7B0C678E290DA835BB26EE826472D66B6A306801E5FE0803C5320C554610A",
                "part_set_header": {
                    "total": 1,
                    "hash": "D0E7B0C678E290DA835BB26EE826472D66B6A306801E5FE0803C5320C554610A"
                }
            },
            "signatures": [
                {
                    "block_id_flag": 2,
                    "validator_address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                    "timestamp": "1970-01-01T00:00:04Z",
                    "signature": "lTGBsjVI6YwIRcxQ6Lct4Q+xrtJc9h3648c42uWe4MpSgy4rUI5g71AEpG90Tbn0PRizjKgCPhokPpQoQLiqAg=="
                }
            ]
        }
    },
    "validator_set": {
        "total_voting_power": "0",
        "validators": [
            {
                "address": "6AE5C701F508EB5B63343858E068C5843F28105F",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "GQEC/HB4sDBAVhHtUzyv4yct9ZGnudaP209QQBSTfSQ="
                },
                "voting_power": "50",
                "proposer_priority": null
            }
        ]
    },
    "next_validator_set": {
        "total_voting_power": "0",
        "validators": [
            {
                "address": "C479DB6F37AB9757035CFBE10B687E27668EE7DF",
                "pub_key": {
                    "type": "tendermint/PubKeyEd25519",
                    "value": "3wf60CidQcsIO7TksXzEZsJefMUFF73k6nP1YeEo9to="
                },
                "voting_power": "50",
                "proposer_priority": null
            }
        ]
    },
    "provider": "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE"
}"#;

parameter_types! {
    pub const MinimumPeriod: u64 = 3;
}

impl pallet_timestamp::Config for TestRuntime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const HeadersToKeep: u32 = 10;
}

impl tendermint_light_client::Config for TestRuntime {
    type Event = Event;
    type HeadersToKeep = HeadersToKeep;
    type TimeProvider = Timestamp;
}

construct_runtime! {
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
        TendermintLightClient: tendermint_light_client::{Pallet, Storage, Event<T>}
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::simple_max(1024);
    pub const TestDbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 25,
        write: 100
    };
}

impl frame_system::Config for TestRuntime {
    type BaseCallFilter = Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = TestDbWeight;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
}

pub fn new_test_ext<T>(test: impl FnOnce() -> T) -> T {
    TestExternalities::new(Default::default()).execute_with(test)
}

fn random_bytes(size: i32) -> Vec<u8> {
    (0..size).map(|_| u8::default()).collect()
}

// TODO : temporary before its used in benchmarking the pallet
#[allow(unused)]
pub fn new_block(
    chain_id_length: i32,
    app_hash_length: i32,
    validators_count: i32,
    validator_name_length: i32,
) -> LightBlockStorage {
    let version = VersionStorage::new(u64::default(), u64::default());
    let chain_id = random_bytes(chain_id_length);
    let height = 3;
    let timestamp = TimestampStorage::new(3, 0);
    let last_block_id = None;
    let last_commit_hash = None;
    let data_hash = None;
    let validators_hash = Hash::default();
    let next_validators_hash = Hash::default();
    let consensus_hash = Hash::default();
    let app_hash = random_bytes(app_hash_length);
    let last_results_hash = None;
    let evidence_hash = None;
    let proposer_address = TendermintAccountId::default();

    let header = HeaderStorage::new(
        version,
        chain_id,
        height,
        timestamp,
        last_block_id,
        last_commit_hash,
        data_hash,
        validators_hash,
        next_validators_hash,
        consensus_hash,
        app_hash,
        last_results_hash,
        evidence_hash,
        proposer_address,
    );

    let height = 3;
    let round = 1;
    let hash = BridgedBlockHash::default();
    let total = u32::default();
    let part_set_header = PartSetHeaderStorage::new(total, hash);
    let block_id = BlockIdStorage::new(hash, part_set_header);

    let signatures = (0..validators_count)
        .map(|_| {
            let validator_address = TendermintAccountId::default();
            let timestamp = TimestampStorage::new(3, 0);
            let signature = TendermintVoteSignature::default();
            CommitSignatureStorage::BlockIdFlagCommit {
                validator_address,
                timestamp,
                signature,
            }
        })
        .collect();

    let commit = CommitStorage::new(height, round, block_id, signatures);
    let signed_header = SignedHeaderStorage::new(header, commit);
    let provider = TendermintPeerId::default();

    let mut total_voting_power = u64::default();

    let validators: Vec<ValidatorInfoStorage> = (0..validators_count)
        .map(|_| {
            let address = TendermintAccountId::default();
            let pub_key = TendermintPublicKey::Ed25519(H256::default());
            let power = u64::default();
            let name = Some(random_bytes(validator_name_length));
            let proposer_priority = i64::default();

            total_voting_power += power;

            ValidatorInfoStorage {
                address,
                pub_key,
                power,
                name,
                proposer_priority,
            }
        })
        .collect();

    let proposer = None;
    let next_validators = validators.clone();

    LightBlockStorage::new(
        signed_header,
        ValidatorSetStorage::new(validators, proposer.clone(), total_voting_power),
        ValidatorSetStorage::new(next_validators, proposer, total_voting_power),
        provider,
    )
}
