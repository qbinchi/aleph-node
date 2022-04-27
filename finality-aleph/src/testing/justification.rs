use std::{cell::RefCell, collections::VecDeque, sync::Arc, time::Duration};

use aleph_bft::{NodeCount, PartialMultisignature, SignatureSet};
use codec::Encode;
use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    Future,
};
use sp_api::BlockId;
use sp_core::Pair;
use sp_runtime::traits::Block;
use tokio::{task::JoinHandle, time::timeout};

use aleph_primitives::{AuthorityPair, AuthoritySignature};
use AcceptancePolicy::*;

use crate::{
    crypto::{Signature, SignatureV1},
    justification::{
        backwards_compatible_decode, AlephJustification, AlephJustificationV1,
        JustificationDecoding, JustificationHandler, JustificationHandlerConfig,
    },
    testing::mocks::{
        create_block, AcceptancePolicy, Client, JustificationRequestSchedulerImpl,
        MockedBlockFinalizer, MockedBlockRequester, SessionInfoProviderImpl, TBlock,
        VerifierWrapper,
    },
    JustificationNotification, SessionPeriod,
};

#[test]
fn correctly_decodes_v1() {
    let mut signature_set: SignatureSet<SignatureV1> = SignatureSet::with_size(7.into());
    for i in 0..7 {
        let id = i.into();
        let signature_v1 = SignatureV1 {
            _id: id,
            sgn: AuthorityPair::generate()
                .0
                .sign(vec![0u8, 0u8, 0u8, 0u8].as_slice()),
        };
        signature_set = signature_set.add_signature(&signature_v1, id);
    }

    let just_v1 = AlephJustificationV1 {
        signature: signature_set,
    };
    let encoded_just: Vec<u8> = just_v1.encode();
    let decoded = backwards_compatible_decode(encoded_just);
    assert_eq!(decoded, JustificationDecoding::V1(just_v1));
}

#[test]
fn correctly_decodes_v2() {
    let mut signature_set: SignatureSet<Signature> = SignatureSet::with_size(7.into());
    for i in 0..7 {
        let authority_signature: AuthoritySignature = AuthorityPair::generate()
            .0
            .sign(vec![0u8, 0u8, 0u8, 0u8].as_slice());
        signature_set = signature_set.add_signature(&authority_signature.into(), i.into());
    }

    let just_v2 = AlephJustification {
        signature: signature_set,
    };
    let encoded_just: Vec<u8> = just_v2.encode();
    let decoded = backwards_compatible_decode(encoded_just);
    assert_eq!(decoded, JustificationDecoding::V2(just_v2));
}

#[test]
fn correctly_decodes_legacy_v1_size4() {
    // This is a justification for 4 nodes generated by the version at commit `a426d7a`
    let raw: Vec<u8> = vec![
        16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 70, 165, 218, 105, 238, 187, 137, 176, 148, 97, 251, 204,
        157, 166, 21, 31, 222, 144, 57, 47, 229, 130, 113, 221, 27, 138, 96, 189, 104, 39, 235,
        217, 107, 217, 184, 99, 252, 227, 142, 169, 60, 92, 64, 26, 128, 73, 40, 49, 208, 54, 173,
        47, 24, 229, 87, 93, 136, 157, 141, 173, 229, 156, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 148, 100,
        171, 132, 5, 223, 228, 210, 92, 49, 165, 58, 241, 100, 3, 208, 81, 194, 122, 36, 4, 31,
        108, 104, 227, 164, 204, 165, 181, 237, 168, 93, 81, 37, 243, 183, 37, 141, 233, 10, 232,
        189, 189, 95, 129, 126, 113, 239, 121, 8, 18, 43, 200, 200, 128, 211, 80, 34, 7, 104, 198,
        215, 213, 8, 1, 2, 0, 0, 0, 0, 0, 0, 0, 126, 125, 118, 133, 4, 152, 203, 42, 36, 177, 160,
        243, 211, 223, 249, 171, 206, 112, 228, 170, 54, 6, 223, 223, 83, 106, 27, 168, 40, 82, 48,
        28, 150, 76, 98, 250, 13, 97, 163, 152, 77, 30, 153, 206, 49, 210, 53, 218, 1, 52, 195, 97,
        58, 229, 250, 198, 35, 155, 118, 249, 180, 123, 12, 8, 0,
    ];
    let decoded = backwards_compatible_decode(raw);
    if let JustificationDecoding::V1(just) = decoded {
        assert_eq!(just.signature.size(), NodeCount(4));
    } else {
        panic!("decoded should be V1, and is {:?}", decoded);
    }
}

#[test]
fn correctly_decodes_legacy_v1_size6() {
    // This is a justification for 6 nodes generated by the version at commit `a426d7a`
    let raw: Vec<u8> = vec![
        24, 1, 0, 0, 0, 0, 0, 0, 0, 0, 82, 120, 213, 50, 242, 152, 25, 224, 232, 243, 218, 52, 111,
        133, 171, 153, 160, 41, 16, 239, 33, 1, 252, 229, 53, 252, 155, 199, 63, 150, 6, 227, 44,
        130, 28, 24, 26, 202, 30, 197, 67, 119, 144, 44, 69, 39, 117, 53, 239, 104, 165, 208, 143,
        204, 4, 165, 6, 165, 27, 134, 7, 44, 172, 7, 1, 1, 0, 0, 0, 0, 0, 0, 0, 173, 204, 199, 231,
        18, 118, 216, 71, 19, 249, 239, 86, 196, 86, 173, 38, 113, 87, 118, 112, 26, 70, 125, 228,
        180, 101, 172, 159, 79, 8, 106, 247, 42, 255, 178, 0, 141, 194, 242, 81, 93, 1, 230, 89,
        247, 247, 233, 237, 136, 9, 254, 103, 74, 37, 43, 124, 226, 59, 146, 242, 143, 208, 49, 13,
        1, 2, 0, 0, 0, 0, 0, 0, 0, 162, 194, 14, 148, 20, 248, 49, 230, 200, 102, 179, 223, 186,
        103, 28, 58, 59, 67, 195, 77, 22, 20, 213, 92, 85, 61, 133, 57, 55, 123, 221, 193, 121, 80,
        18, 68, 92, 5, 2, 56, 55, 43, 1, 214, 145, 131, 146, 103, 245, 135, 25, 251, 212, 85, 230,
        223, 143, 44, 190, 102, 121, 121, 67, 12, 1, 3, 0, 0, 0, 0, 0, 0, 0, 176, 17, 161, 159, 68,
        184, 2, 127, 122, 162, 2, 213, 232, 113, 111, 86, 35, 196, 150, 186, 221, 188, 14, 245, 41,
        21, 206, 174, 134, 142, 191, 212, 20, 102, 99, 111, 110, 48, 75, 214, 163, 173, 107, 251,
        82, 24, 43, 131, 210, 160, 59, 88, 111, 150, 236, 25, 128, 36, 179, 255, 56, 189, 99, 13,
        1, 4, 0, 0, 0, 0, 0, 0, 0, 140, 68, 206, 82, 199, 166, 235, 142, 114, 218, 219, 235, 206,
        76, 253, 180, 143, 213, 7, 39, 49, 154, 83, 142, 250, 26, 74, 37, 95, 106, 51, 179, 185,
        75, 63, 244, 63, 1, 179, 125, 53, 110, 220, 13, 126, 46, 124, 173, 98, 164, 194, 175, 52,
        108, 43, 68, 94, 254, 77, 39, 172, 255, 145, 10, 0,
    ];
    let decoded = backwards_compatible_decode(raw);
    if let JustificationDecoding::V1(just) = decoded {
        assert_eq!(just.signature.size(), NodeCount(6));
    } else {
        panic!("decoded should be V1, and is {:?}", decoded);
    }
}

const SESSION_PERIOD: SessionPeriod = SessionPeriod(5u32);
const FINALIZED_HEIGHT: u64 = 22;

type TJustHandler = JustificationHandler<
    TBlock,
    VerifierWrapper,
    MockedBlockRequester,
    Client,
    JustificationRequestSchedulerImpl,
    SessionInfoProviderImpl,
    MockedBlockFinalizer,
>;
type Sender = UnboundedSender<JustificationNotification<TBlock>>;
type Environment = (
    TJustHandler,
    Client,
    MockedBlockRequester,
    MockedBlockFinalizer,
    JustificationRequestSchedulerImpl,
);

fn create_justification_notification_for(block: TBlock) -> JustificationNotification<TBlock> {
    JustificationNotification {
        justification: AlephJustification {
            signature: SignatureSet::with_size(0.into()),
        },
        hash: block.hash(),
        number: block.header.number,
    }
}

fn run_justification_handler(
    justification_handler: TJustHandler,
) -> (JoinHandle<()>, Sender, Sender) {
    let (auth_just_tx, auth_just_rx) = unbounded();
    let (imp_just_tx, imp_just_rx) = unbounded();

    let handle =
        tokio::spawn(async move { justification_handler.run(auth_just_rx, imp_just_rx).await });

    (handle, auth_just_tx, imp_just_tx)
}

fn prepare_env(
    finalization_height: u64,
    verification_policy: AcceptancePolicy,
    request_policy: AcceptancePolicy,
) -> Environment {
    let client = Client::new(finalization_height);
    let info_provider = SessionInfoProviderImpl::new(SESSION_PERIOD, verification_policy);
    let finalizer = MockedBlockFinalizer::new();
    let requester = MockedBlockRequester::new();
    let config = JustificationHandlerConfig::test();
    let justification_request_scheduler = JustificationRequestSchedulerImpl::new(request_policy);

    let justification_handler = JustificationHandler::new(
        info_provider,
        requester.clone(),
        Arc::new(client.clone()),
        finalizer.clone(),
        justification_request_scheduler.clone(),
        None,
        config,
    );

    (
        justification_handler,
        client,
        requester,
        finalizer,
        justification_request_scheduler,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn panics_and_stops_when_authority_channel_is_closed() {
    let justification_handler = prepare_env(1u64, AlwaysReject, AlwaysReject).0;
    let (handle, auth_just_tx, _) = run_justification_handler(justification_handler);
    auth_just_tx.close_channel();

    let handle = async move { handle.await.unwrap_err() };
    match timeout(Duration::from_millis(50), handle).await {
        Ok(err) => assert!(err.is_panic()),
        Err(_) => panic!("JustificationHandler did not stop!"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn panics_and_stops_when_import_channel_is_closed() {
    let justification_handler = prepare_env(1u64, AlwaysReject, AlwaysReject).0;
    let (handle, _, imp_just_tx) = run_justification_handler(justification_handler);
    imp_just_tx.close_channel();

    let handle = async move { handle.await.unwrap_err() };
    match timeout(Duration::from_millis(50), handle).await {
        Ok(err) => assert!(err.is_panic()),
        Err(_) => panic!("JustificationHandler did not stop!"),
    }
}

async fn run_test<F, S>(env: Environment, scenario: S)
where
    F: Future,
    S: FnOnce(
        Sender,
        Sender,
        Client,
        MockedBlockRequester,
        MockedBlockFinalizer,
        JustificationRequestSchedulerImpl,
    ) -> F,
{
    let (justification_handler, client, requester, finalizer, justification_request_scheduler) =
        env;
    let (handle_run, auth_just_tx, imp_just_tx) = run_justification_handler(justification_handler);
    scenario(
        auth_just_tx.clone(),
        imp_just_tx.clone(),
        client,
        requester,
        finalizer,
        justification_request_scheduler,
    )
    .await;
    auth_just_tx.close_channel();
    imp_just_tx.close_channel();
    let _ = timeout(Duration::from_millis(10), handle_run).await;
}

async fn expect_finalized(
    finalizer: &MockedBlockFinalizer,
    justification_request_scheduler: &JustificationRequestSchedulerImpl,
    block: TBlock,
) {
    assert!(finalizer.has_been_invoked_with(block).await);
    assert!(justification_request_scheduler.has_been_finalized().await);
}

async fn expect_not_finalized(
    finalizer: &MockedBlockFinalizer,
    justification_request_scheduler: &JustificationRequestSchedulerImpl,
) {
    assert!(finalizer.has_not_been_invoked().await);
    assert!(!justification_request_scheduler.has_been_finalized().await);
}

async fn expect_requested(
    requester: &MockedBlockRequester,
    justification_request_scheduler: &JustificationRequestSchedulerImpl,
    block: TBlock,
) {
    assert!(requester.has_been_invoked_with(block).await);
    assert!(justification_request_scheduler.has_been_requested().await);
}

async fn expect_not_requested(
    requester: &MockedBlockRequester,
    justification_request_scheduler: &JustificationRequestSchedulerImpl,
) {
    assert!(requester.has_not_been_invoked().await);
    assert!(!justification_request_scheduler.has_been_requested().await);
}

#[tokio::test(flavor = "multi_thread")]
async fn leads_to_finalization_when_appropriate_justification_comes() {
    run_test(
        prepare_env(FINALIZED_HEIGHT, AlwaysAccept, AlwaysReject),
        |_, imp_just_tx, client, _, finalizer, justification_request_scheduler| async move {
            let block = client.next_block_to_finalize();
            let message = create_justification_notification_for(block.clone());
            imp_just_tx.unbounded_send(message).unwrap();
            expect_finalized(&finalizer, &justification_request_scheduler, block).await;
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn waits_for_verifier_before_finalizing() {
    let verification_policy = FromSequence(RefCell::new(VecDeque::from(vec![false, false, true])));
    run_test(
        prepare_env(FINALIZED_HEIGHT, verification_policy, AlwaysReject),
        |_, imp_just_tx, client, _, finalizer, justification_request_scheduler| async move {
            let block = client.next_block_to_finalize();
            let message = create_justification_notification_for(block.clone());

            imp_just_tx.unbounded_send(message.clone()).unwrap();
            expect_not_finalized(&finalizer, &justification_request_scheduler).await;

            imp_just_tx.unbounded_send(message.clone()).unwrap();
            expect_not_finalized(&finalizer, &justification_request_scheduler).await;

            imp_just_tx.unbounded_send(message).unwrap();
            expect_finalized(&finalizer, &justification_request_scheduler, block).await;
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn keeps_finalizing_block_if_not_finalized_yet() {
    run_test(
        prepare_env(FINALIZED_HEIGHT, AlwaysAccept, AlwaysReject),
        |auth_just_tx, imp_just_tx, client, _, finalizer, justification_request_scheduler| async move {
            let block = client.next_block_to_finalize();
            let message = create_justification_notification_for(block.clone());

            imp_just_tx.unbounded_send(message.clone()).unwrap();
            expect_finalized(&finalizer, &justification_request_scheduler, block.clone()).await;

            auth_just_tx.unbounded_send(message).unwrap();
            expect_finalized(&finalizer, &justification_request_scheduler, block).await;
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn ignores_notifications_for_old_blocks() {
    run_test(
        prepare_env(FINALIZED_HEIGHT, AlwaysAccept, AlwaysReject),
        |_, imp_just_tx, client, _, finalizer, justification_request_scheduler| async move {
            let block = client.get_block(BlockId::Number(1u64)).unwrap();
            let message = create_justification_notification_for(block);
            imp_just_tx.unbounded_send(message).unwrap();
            expect_not_finalized(&finalizer, &justification_request_scheduler).await;
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn ignores_notifications_from_future_session() {
    run_test(
        prepare_env(FINALIZED_HEIGHT, AlwaysAccept, AlwaysReject),
        |_, imp_just_tx, _, _, finalizer, justification_request_scheduler| async move {
            let block = create_block([1u8; 32].into(), FINALIZED_HEIGHT + SESSION_PERIOD.0 as u64);
            let message = create_justification_notification_for(block);
            imp_just_tx.unbounded_send(message).unwrap();
            expect_not_finalized(&finalizer, &justification_request_scheduler).await;
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn does_not_buffer_notifications_from_future_session() {
    run_test(
        prepare_env((SESSION_PERIOD.0 - 2) as u64, AlwaysAccept, AlwaysReject),
        |_, imp_just_tx, client, _, finalizer, justification_request_scheduler| async move {
            let current_block = client.next_block_to_finalize();
            let future_block = create_block(current_block.hash(), SESSION_PERIOD.0 as u64);

            let message = create_justification_notification_for(future_block);
            imp_just_tx.unbounded_send(message).unwrap();
            expect_not_finalized(&finalizer, &justification_request_scheduler).await;

            let message = create_justification_notification_for(current_block.clone());
            imp_just_tx.unbounded_send(message).unwrap();
            expect_finalized(&finalizer, &justification_request_scheduler, current_block).await;

            expect_not_finalized(&finalizer, &justification_request_scheduler).await;
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn requests_for_session_ending_justification() {
    run_test(
        prepare_env((SESSION_PERIOD.0 - 2) as u64, AlwaysReject, AlwaysAccept),
        |_, imp_just_tx, client, requester, _, justification_request_scheduler| async move {
            let last_block = client.next_block_to_finalize();

            // doesn't need any notification passed to keep asking
            expect_requested(
                &requester,
                &justification_request_scheduler,
                last_block.clone(),
            )
            .await;
            expect_requested(
                &requester,
                &justification_request_scheduler,
                last_block.clone(),
            )
            .await;

            // asks also after processing some notifications
            let message = create_justification_notification_for(last_block.clone());
            imp_just_tx.unbounded_send(message).unwrap();

            expect_requested(&requester, &justification_request_scheduler, last_block).await;
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn does_not_request_for_session_ending_justification_too_often() {
    run_test(
        prepare_env((SESSION_PERIOD.0 - 2) as u64, AlwaysReject, AlwaysReject),
        |_, _, client, requester, _, justification_request_scheduler| async move {
            expect_not_requested(&requester, &justification_request_scheduler).await;

            justification_request_scheduler.update_policy(AlwaysAccept);
            expect_requested(
                &requester,
                &justification_request_scheduler,
                client.next_block_to_finalize(),
            )
            .await;

            justification_request_scheduler.update_policy(AlwaysReject);
            expect_not_requested(&requester, &justification_request_scheduler).await;
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn does_not_request_nor_finalize_when_verifier_is_not_available() {
    run_test(
        prepare_env((SESSION_PERIOD.0 - 2) as u64, Unavailable, AlwaysAccept),
        |_, imp_just_tx, client, requester, finalizer, justification_request_scheduler| async move {
            expect_not_requested(&requester, &justification_request_scheduler).await;

            let block = client.next_block_to_finalize();
            imp_just_tx
                .unbounded_send(create_justification_notification_for(block))
                .unwrap();

            expect_not_finalized(&finalizer, &justification_request_scheduler).await;
            expect_not_requested(&requester, &justification_request_scheduler).await;
        },
    )
    .await;
}
