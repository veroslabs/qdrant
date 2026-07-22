use std::sync::Arc;

use tokio::sync::mpsc;

use crate::operations::types::CollectionResult;
use crate::optimizers_builder::{OptimizersConfig, build_optimizers};
use crate::shards::local_shard::LocalShard;
use crate::update_handler::UpdateSignal;

impl LocalShard {
    pub fn trigger_optimizers(&self) {
        // Send a trigger signal and ignore errors because all error cases are acceptable:
        // - If receiver is already dead - we do not care
        // - If channel is full - optimization will be triggered by some other signal
        let _ = self.update_sender.load().try_send(UpdateSignal::Nop);
    }

    pub async fn stop_flush_worker(&self) {
        let mut update_handler = self.update_handler.lock().await;
        update_handler.stop_flush_worker()
    }

    pub async fn wait_update_workers_stop(&self) -> CollectionResult<()> {
        let mut update_handler = self.update_handler.lock().await;
        update_handler.wait_workers_stops().await
    }

    /// Handles updates to the optimizer configuration by rebuilding optimizers
    /// and restarting the update handler's workers with the new configuration.
    ///
    /// ## Cancel safety
    ///
    /// This function is **not** cancel safe.
    pub async fn on_optimizer_config_update(
        &self,
        effective_optimizers_config: &OptimizersConfig,
    ) -> CollectionResult<()> {
        let config = self.collection_config.read().await;
        let mut update_handler = self.update_handler.lock().await;

        let (update_sender, update_receiver) =
            mpsc::channel(self.shared_storage_config.update_queue_size);
        // makes sure that the Stop signal is the last one in this channel
        let old_sender = self.update_sender.swap(Arc::new(update_sender));
        old_sender.send(UpdateSignal::Stop).await?;
        update_handler.stop_flush_worker();

        update_handler.wait_workers_stops().await?;
        let new_optimizers = build_optimizers(
            &self.path,
            &config.params,
            effective_optimizers_config,
            &config.hnsw_config,
            &self.shared_storage_config.hnsw_global_config,
            &config.quantization_config,
        );
        update_handler.optimizers = new_optimizers;
        update_handler.flush_interval_sec = effective_optimizers_config.flush_interval_sec;
        update_handler.max_optimization_threads =
            effective_optimizers_config.max_optimization_threads;
        update_handler.run_workers(update_receiver);

        self.update_sender.load().send(UpdateSignal::Nop).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common::budget::ResourceBudget;
    use common::save_on_disk::SaveOnDisk;
    use tempfile::Builder;
    use tokio::runtime::Handle;
    use tokio::sync::RwLock;

    use super::LocalShard;
    use crate::collection::payload_index_schema::PayloadIndexSchema;
    use crate::shards::shard_trait::ShardOperation;
    use crate::tests::fixtures::create_collection_config;

    #[tokio::test(flavor = "multi_thread")]
    async fn update_handler_uses_effective_optimizer_config() {
        let collection_dir = Builder::new().prefix("test_collection").tempdir().unwrap();
        let payload_index_schema_dir = Builder::new().prefix("qdrant-test").tempdir().unwrap();
        let payload_index_schema = Arc::new(
            SaveOnDisk::<PayloadIndexSchema>::load_or_init_default(
                payload_index_schema_dir.path().join("payload-schema.json"),
            )
            .unwrap(),
        );

        let mut config = create_collection_config();
        config.optimizer_config.max_optimization_threads = Some(0);
        let mut effective_optimizers_config = config.optimizer_config.clone();
        effective_optimizers_config.max_optimization_threads = Some(1);
        let collection_config = Arc::new(RwLock::new(config));

        let runtime = Handle::current();
        let shard = LocalShard::build(
            0,
            "test".to_string(),
            collection_dir.path(),
            collection_config.clone(),
            Arc::new(Default::default()),
            payload_index_schema,
            runtime.clone(),
            runtime,
            ResourceBudget::default(),
            effective_optimizers_config.clone(),
        )
        .await
        .unwrap();

        let initial_max_threads = shard.update_handler.lock().await.max_optimization_threads;
        shard
            .on_optimizer_config_update(&effective_optimizers_config)
            .await
            .unwrap();
        let updated_max_threads = shard.update_handler.lock().await.max_optimization_threads;
        let persisted_max_threads = collection_config
            .read()
            .await
            .optimizer_config
            .max_optimization_threads;
        shard.stop_gracefully().await;

        assert_eq!(
            (
                initial_max_threads,
                updated_max_threads,
                persisted_max_threads,
            ),
            (Some(1), Some(1), Some(0)),
        );
    }
}
