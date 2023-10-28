use log::info;
use mongodb::bson::oid::ObjectId;
use rust_sdk::{
    api::{artifact, host, job_execution},
    model::{
        artifact::{ArtifactStatus, UpdateArtifactDTO},
        host::{HostStatus, UpdateHostDTO},
        job_execution::{JobExecutionStatus, UpdateJobExecutionDTO},
    },
};

use crate::config::config::AgentConfig;

pub async fn set_host_status(config: AgentConfig, id: ObjectId, status: HostStatus) {
    info!("Updating host status to \"{:?}\"", status);

    host::update(
        config.sdk_config.clone(),
        id,
        UpdateHostDTO {
            status: Some(status),
        },
    )
    .await;
}

pub async fn set_job_execution_status(
    config: AgentConfig,
    id: ObjectId,
    status: JobExecutionStatus,
) {
    info!("Updating job execution status to \"{:?}\"", status);

    job_execution::update(
        config.sdk_config,
        id,
        UpdateJobExecutionDTO {
            host_id: None,
            output_artifacts: None,
            status: Some(status),
        },
    )
    .await;
}

pub async fn set_artifact_status(config: AgentConfig, id: ObjectId, status: ArtifactStatus) {
    info!("Updating artifact status to \"{:?}\"", status);

    artifact::update(
        config.sdk_config,
        id,
        UpdateArtifactDTO {
            status: Some(status),
        },
    )
    .await;
}
