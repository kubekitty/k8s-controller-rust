use futures::StreamExt;
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::Pod,
    networking::v1::NetworkPolicy,
    rbac::v1::Role,
};
use kube::{
    api::{Api, ListParams, WatchEvent, WatchParams},
    Client, Resource,
};
use serde::de::DeserializeOwned;
use tokio::sync::mpsc;
use std::sync::Arc;
use tracing::{info, error};

use crate::rules::RulesEngine;
use crate::types::Finding;

pub struct Controller {
    client: Client,
    rules_engine: Arc<RulesEngine>,
    findings_tx: mpsc::Sender<Finding>,
}

impl Controller {
    pub async fn new(client: Client, rules_engine: RulesEngine) -> Self {
        let (findings_tx, mut findings_rx) = mpsc::channel(100);
        
        // Spawn findings processor
        tokio::spawn(async move {
            while let Some(finding) = findings_rx.recv().await {
                info!("Finding: {:?}", finding);
            }
        });

        Self {
            client,
            rules_engine: Arc::new(rules_engine),
            findings_tx,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pod_watcher = self.watch_resource::<Pod>("pods");
        let deployment_watcher = self.watch_resource::<Deployment>("deployments");
        let network_watcher = self.watch_resource::<NetworkPolicy>("networkpolicies");
        let role_watcher = self.watch_resource::<Role>("roles");

        futures::join!(
            pod_watcher,
            deployment_watcher,
            network_watcher,
            role_watcher
        );

        Ok(())
    }

    async fn watch_resource<K>(&self, resource_type: &str) -> Result<(), Box<dyn std::error::Error>>
    where
        K: Resource + Clone + DeserializeOwned + std::fmt::Debug + Send + 'static + serde::Serialize,
        K::DynamicType: Default,
    {
        let api: Api<K> = Api::all(self.client.clone());
        let watch_params = WatchParams::default();
        
        let mut stream = api.watch(&watch_params, "0").await?.boxed();
        
        while let Some(event) = stream.next().await {
            match event {
                Ok(WatchEvent::Added(obj)) | Ok(WatchEvent::Modified(obj)) => {
                    self.audit_resource(&obj, resource_type).await?;
                }
                Ok(WatchEvent::Deleted(_)) => {
                    info!("Resource deleted: {}", resource_type);
                }
                Ok(WatchEvent::Bookmark(_)) => {}
                Ok(WatchEvent::Error(e)) => {
                    error!("Error in watch stream: {:?}", e);
                }
                Err(e) => {
                    error!("Watch error: {:?}", e);
                }
            }
        }

        Ok(())
    }

    async fn audit_resource<K>(&self, resource: &K, resource_type: &str) -> Result<(), Box<dyn std::error::Error>>
    where
        K: std::fmt::Debug + serde::Serialize,
    {
        let findings = self.rules_engine.evaluate_rules(resource_type, resource);
        
        for finding in findings {
            if let Err(e) = self.findings_tx.send(finding).await {
                error!("Failed to send finding: {:?}", e);
            }
        }

        Ok(())
    }
}