use super::types::*;
use core::panic;
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::Arc;
use sui_protocol_config::ProtocolConfig;
use sui_single_node_benchmark::benchmark_context::BenchmarkContext;
use sui_single_node_benchmark::mock_storage::InMemoryObjectStore;
use sui_types::base_types::{ObjectID, ObjectRef};
use sui_types::digests::TransactionDigest;
use sui_types::effects::TransactionEffectsAPI;
use sui_types::executable_transaction::VerifiedExecutableTransaction;
use sui_types::message_envelope::Message;
use sui_types::messages_checkpoint::CheckpointDigest;
use sui_types::object::Object;
use sui_types::sui_system_state::SuiSystemStateTrait;
use sui_types::transaction::{TransactionDataAPI, VerifiedCertificate};
use tokio::sync::mpsc;
use tokio::time::Duration;

/*****************************************************************************************
 *                                    PreExec Worker                                   *
 *****************************************************************************************/

pub struct PreExecWorkerState {
    pub memory_store: Arc<InMemoryObjectStore>,
    pub context: Arc<BenchmarkContext>,
    pub ready_txs: DashMap<TransactionDigest, ()>,
    pub waiting_child_objs: DashMap<TransactionDigest, HashSet<ObjectID>>,
    pub received_objs: DashMap<TransactionDigest, Vec<Option<(ObjectRef, Object)>>>,
    pub received_child_objs: DashMap<TransactionDigest, Vec<Option<(ObjectRef, Object)>>>,
    pub locked_exec_count: DashMap<TransactionDigest, u8>,
    pub genesis_digest: CheckpointDigest,
}

impl PreExecWorkerState {
    pub fn new(
        new_store: InMemoryObjectStore,
        genesis_digest: CheckpointDigest,
        ctx: Arc<BenchmarkContext>,
    ) -> Self {
        Self {
            memory_store: Arc::new(new_store),
            context: ctx,
            ready_txs: DashMap::new(),
            waiting_child_objs: DashMap::new(),
            received_objs: DashMap::new(),
            received_child_objs: DashMap::new(),
            locked_exec_count: DashMap::new(),
            genesis_digest,
        }
    }

    async fn async_exec(
        full_tx: TransactionWithEffects,
        memstore: Arc<InMemoryObjectStore>,
        protocol_config: &ProtocolConfig,
        reference_gas_price: u64,
        ctx: Arc<BenchmarkContext>,
        in_buffer: &mpsc::UnboundedSender<TransactionWithResults>,
    ) {
        let tx = full_tx.tx.clone();

        // let effect = ctx.validator().execute_raw_transaction(tx).await;

        //ctx.validator().execute_dry_run(tx).await

        let input_objects = tx.transaction_data().input_objects().unwrap();
        // FIXME: ugly deref
        let objects = memstore
            .read_objects_for_execution(
                &**(ctx.validator().get_epoch_store()),
                &tx.key(),
                &input_objects,
            )
            .unwrap();

        let executable = VerifiedExecutableTransaction::new_from_certificate(
            VerifiedCertificate::new_unchecked(tx),
        );

        let _validator = ctx.validator();
        let (gas_status, input_objects) = sui_transaction_checks::check_certificate_input(
            &executable,
            objects,
            protocol_config,
            reference_gas_price,
        )
        .unwrap();
        let (kind, signer, gas) = executable.transaction_data().execution_parts();
        let (inner_temp_store, _, effects, _) = ctx
            .validator()
            .get_epoch_store()
            .executor()
            .execute_transaction_to_effects(
                &memstore,
                protocol_config,
                ctx.validator()
                    .get_validator()
                    .metrics
                    .limits_metrics
                    .clone(),
                false,
                &HashSet::new(),
                &ctx.validator().get_epoch_store().epoch(),
                0,
                input_objects,
                gas,
                gas_status,
                kind,
                signer,
                *executable.digest(),
            );
        assert!(effects.status().is_ok());

        let tx_res = TransactionWithResults {
            tx_effects: effects,
            written: inner_temp_store.written.clone(),
        };

        memstore.commit_objects(inner_temp_store);
        println!("finish exec a txn");

        if let Err(e) = in_buffer.send(tx_res) {
            eprintln!("PRE failed to forward in-channel exec res: {:?}", e);
        }
    }

    pub async fn run(
        &mut self,
        _tx_count: u64,
        _duration: Duration,
        in_channel: &mut mpsc::Receiver<NetworkMessage>,
        out_channel: &mpsc::Sender<NetworkMessage>,
        my_id: u16,
    ) {
        let mut consensus_interval = tokio::time::interval(Duration::from_millis(100));
        let (in_buffer, mut out_buffer) = mpsc::unbounded_channel::<TransactionWithResults>();

        loop {
            tokio::select! {
                Some(msg) = in_channel.recv() => {
                    println!("{} receive a txn", my_id);
                    let msg = msg.payload;
                    if let RemoraMessage::ProposeExec(full_tx) = msg {
                        let memstore = self.memory_store.clone();
                        let context = self.context.clone();
                        let in_buffer = in_buffer.clone();
                        tokio::spawn(async move {
                            Self::async_exec(full_tx.clone(),
                                             memstore,
                                             context.validator().get_epoch_store().protocol_config(),
                                             context.validator().get_epoch_store().reference_gas_price(),
                                             context,
                                             &in_buffer,
                            ).await
                        });
                    } else {
                        eprintln!("EW {} received unexpected message from: {:?}", my_id, msg);
                        panic!("unexpected message");
                    };
                },

                _ = consensus_interval.tick() => {
                    // drain the exec results and send it out
                    while let Ok(msg) = out_buffer.try_recv() {
                        out_channel.send(NetworkMessage {
                        src: my_id,
                        dst: vec![1],
                        payload: RemoraMessage::PreExecResult(msg),
                        })
                        .await
                        .expect("sending failed");
                    }

                },
            }
        }
    }
}
