use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use sui_single_node_benchmark::{
    benchmark_context::BenchmarkContext, command::Component, workload::Workload,
};
use tokio::sync::mpsc;

use super::agents::*;
use crate::{
    metrics::Metrics,
    pre_exec_worker::{self},
    tx_gen_agent::WORKLOAD,
    types::*,
};

pub struct PreExecAgent {
    id: UniqueId,
    in_channel: mpsc::Receiver<NetworkMessage>,
    out_channel: mpsc::Sender<NetworkMessage>,
    attrs: GlobalConfig,
    metrics: Arc<Metrics>,
}

pub const COMPONENT: Component = Component::Baseline;

#[async_trait]
impl Agent for PreExecAgent {
    fn new(
        id: UniqueId,
        in_channel: mpsc::Receiver<NetworkMessage>,
        out_channel: mpsc::Sender<NetworkMessage>,
        attrs: GlobalConfig,
        metrics: Arc<Metrics>,
    ) -> Self {
        PreExecAgent {
            id,
            in_channel,
            out_channel,
            attrs,
            metrics,
        }
    }

    async fn run(&mut self) {
        println!("Starting PreExec agent {}", self.id);

        let my_attrs = &self.attrs.get(&self.id).unwrap().attrs;
        let tx_count = my_attrs.get("tx_count").unwrap().parse().unwrap();

        let duration_secs = my_attrs["duration"].parse::<u64>().unwrap();
        let duration = Duration::from_secs(duration_secs);

        let workload = Workload::new(tx_count * duration.as_secs(), WORKLOAD);
        let context: Arc<BenchmarkContext> = {
            let ctx = BenchmarkContext::new(workload.clone(), COMPONENT, true).await;
            Arc::new(ctx)
        };

        let store = context.validator().create_in_memory_store();

        let mut pre_exec_state = pre_exec_worker::PreExecWorkerState::new(store, context.clone());
        pre_exec_state
            .run(
                tx_count,
                duration,
                &mut self.in_channel,
                &self.out_channel,
                self.id,
                self.metrics.clone(),
            )
            .await;
    }
}
