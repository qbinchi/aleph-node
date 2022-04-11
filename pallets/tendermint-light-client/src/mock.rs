use crate::{
    self as tendermint_light_client,
    types::{
        LightBlockStorage, TendermintPeerId, TimestampStorage, ValidatorInfoStorage,
        ValidatorSetStorage,
    },
};
use frame_support::{
    construct_runtime, parameter_types, sp_io, traits::Everything, weights::RuntimeDbWeight,
};
use primitives::TENDERMINT_MAX_VOTES_COUNT;
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::vec::Vec;
use tendermint_testgen as testgen;

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
    pub const HeadersToKeep: u32 = 3;
    pub const MaxVotesCount: u32 = TENDERMINT_MAX_VOTES_COUNT;
}

impl tendermint_light_client::Config for TestRuntime {
    type Event = Event;
    type HeadersToKeep = HeadersToKeep;
    type TimeProvider = Timestamp;
    type MaxVotesCount = MaxVotesCount;
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

#[cfg(feature = "runtime-benchmarks")]
pub fn new_genesis_storage() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<TestRuntime>()
        .unwrap()
        .into()
}

// #[cfg(feature = "runtime-benchmarks")]
pub fn generate_consecutive_blocks(
    n: usize,
    chain_id: String,
    validators_count: i32,
    from_height: u64,
    from_timestamp: TimestampStorage,
) -> Vec<LightBlockStorage> {
    let validators = (0..validators_count)
        .map(|id| testgen::Validator::new(&id.to_string()).voting_power(50))
        .collect::<Vec<testgen::Validator>>();

    let header = testgen::Header::new(&validators)
        .height(from_height)
        .chain_id(&chain_id)
        .next_validators(&validators)
        .time(from_timestamp.try_into().unwrap());

    let commit = testgen::Commit::new(header.clone(), 1);

    let validators = testgen::validator::generate_validators(&validators)
        .unwrap()
        .into_iter()
        .map(|v| v.try_into().unwrap())
        .collect::<Vec<ValidatorInfoStorage>>();

    let validators_set = ValidatorSetStorage::new(validators, None, 50 * validators_count as u64);

    let signed_header = testgen::light_block::generate_signed_header(&header, &commit).unwrap();

    let default_provider: TendermintPeerId =
        "BADFADAD0BEFEEDC0C0ADEADBEEFC0FFEEFACADE".parse().unwrap();

    let mut block = testgen::LightBlock::new(header, commit);
    let mut blocks = Vec::with_capacity(n);

    let block_storage = LightBlockStorage::new(
        signed_header.clone().try_into().unwrap(),
        validators_set.clone(),
        validators_set.clone(),
        default_provider,
    );

    // println!(
    //     "init block/block_hash {:#?}",
    //     &signed_header.clone().commit.block_id.hash
    // );
    // println!(
    //     "init block storage/block_hash {:#?}",
    //     &block_storage.clone().signed_header.commit.block_id.hash
    // );

    // println!();

    // println!(
    //     "init block/last_block_id_hash {:#?}",
    //     &block.clone().header.unwrap().last_block_id_hash
    // );
    // println!(
    //     "init block storage/last_block_id_hash {:#?}",
    //     &block_storage.clone().signed_header.header.last_block_id
    // );

    blocks.push(block_storage);

    for _index in 1..n {
        block = block.next();

        let testgen::LightBlock { header, commit, .. } = block.clone();

        println!(
            "header.time: {:?} hash: {:?}",
            &header.clone().unwrap().time,
            &header.clone().unwrap().last_block_id_hash
        );

        let signed_header = testgen::light_block::generate_signed_header(
            &header.clone().unwrap(),
            &commit.unwrap(),
        )
        .unwrap();

        let bs = LightBlockStorage::new(
            signed_header.try_into().unwrap(),
            validators_set.clone(),
            validators_set.clone(),
            default_provider,
        );

        // if _index == 1 {
        // println!(
        //     "next block / last_block_id_hash {:#?}",
        //     &b.clone().header.unwrap().last_block_id_hash
        // );
        // println!(
        //     "next block storage / last_block_id_hash {:#?}",
        //     &bs.clone().signed_header.header.last_block_id
        // );

        // println!("{:#?}", &b.header.clone());
        // println!("{:#?}", &bs.signed_header.header);
        // }

        blocks.push(bs);
    }

    blocks.reverse();

    // TODO
    blocks.iter().for_each(|b| {
        println!("TIME {:?}", b.signed_header.header.timestamp);

        // println!(
        //     " last_block_id {:?}\n last_commit_hash {:?} \n header_hash {:?}\n part_set_header_hash {:?}\n",
        //     b.signed_header.header.last_block_id,
        //     b.signed_header.header.last_commit_hash,
        //     b.signed_header.commit.block_id.hash,
        //     b.signed_header.commit.block_id.part_set_header.hash
        // );

        // println!("{:#?}", b);
    });

    // println!("# blocks  {}", blocks.len());

    blocks
}