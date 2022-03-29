#![cfg(test)]
extern crate test;

use frame_election_provider_support::{ElectionProvider, Support};

use crate::mock::*;

#[test]
fn test_elect() {
    new_test_ext(vec![1, 2]).execute_with(|| {
        let elected = <Elections as ElectionProvider<AccountId, u64>>::elect();
        assert!(elected.is_ok());

        let supp = Support {
            total: 0,
            voters: Vec::new(),
        };

        assert_eq!(elected.unwrap(), &[(1, supp.clone()), (2, supp)]);
    });
}

use test::Bencher;

fn init_voters(nominators_per_validator: u64) {
    unsafe {
        TARGETS = (0..10u64)
            .map(|i| (0..nominators_per_validator).map(move |n| (n, 10u64, vec![i])))
            .flatten()
            .collect()
    }
}

fn run_elect_bench(nominators_per_validator: u64, b: &mut Bencher) {
    new_test_ext((0..10).collect()).execute_with(|| {
        init_voters(nominators_per_validator);
        b.iter(|| {
            let res = <Elections as ElectionProvider<AccountId, u64>>::elect();
            let support = &res.unwrap()[0].1;
            assert!(support.voters.len() == nominators_per_validator as usize);
        });
    });
}

#[bench]
fn bench_elect_5k(b: &mut Bencher) {
    run_elect_bench(500, b)
}

#[bench]
fn bench_elect_10k(b: &mut Bencher) {
    run_elect_bench(1000, b)
}
