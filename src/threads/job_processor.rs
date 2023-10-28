use log::info;
use mongodb::bson::oid::ObjectId;
use rust_sdk::api::{job, runtime};
use rust_sdk::model::common::PlatformExecutionType;
use rust_sdk::model::host::HostStatus;
use rust_sdk::model::job_execution::JobExecutionStatus;
use std::fs;

use crate::config::config::AgentConfig;
use crate::operations::prepare_input_artifacts::prepare_input_artifacts;
use crate::operations::prepare_output_artifacts::prepare_output_artifacts;
use crate::operations::prepare_runtime::prepare_runtime;
use crate::operations::run_wasmer::run_wasmer;
use crate::operations::update_status::{set_host_status, set_job_execution_status};
use crate::operations::wait_for_job_execution::wait_for_job_execution;

pub async fn job_processor(config: AgentConfig, host_id: ObjectId) {
    loop {
        // Update host status to Idle
        set_host_status(config.clone(), host_id.clone(), HostStatus::Idle).await;

        // Wait until available job execution
        let job_execution = wait_for_job_execution(config.clone(), host_id.clone()).await;
        let job = job::get(config.clone().sdk_config, job_execution.job_id).await;
        let runtime = runtime::get(config.clone().sdk_config, job.runtime_id).await;

        // Update host status to "Busy"
        set_host_status(config.clone(), host_id.clone(), HostStatus::Busy).await;

        // Update job execution status to "Execution"
        set_job_execution_status(
            config.clone(),
            job_execution.id.clone(),
            JobExecutionStatus::Execution,
        )
        .await;

        // Create temporary "/<Job Execution ID>/input" and "/<Job Execution ID>/output" directories
        info!("Creating temporary input and output directories");
        fs::create_dir_all(
            config.get_job_execution_input_path(&job_execution.id.to_string().as_str()),
        )
        .expect("could not create dir");
        fs::create_dir_all(
            config.get_job_execution_output_path(&job_execution.id.to_string().as_str()),
        )
        .expect("could not create dir");

        // Download runtime to temporary working directory
        info!("Downloading runtime");
        prepare_runtime(config.clone(), job_execution.id.clone(), job.runtime_id).await;

        // Download input artifacts to "/<Job Execution ID>/input" directory
        info!("Downloading input artifacts");
        prepare_input_artifacts(config.clone(), job_execution.id, job.input_artifacts).await;

        // Run job execution
        info!("Running job execution");

        match runtime.platform_execution_type {
            PlatformExecutionType::Docker => (),
            PlatformExecutionType::BareMetal => (),
            PlatformExecutionType::Wasmer => {
                run_wasmer(config.clone(), job_execution.id, runtime.id)
            }
        }

        // Update job execution status to "PendingArtifactUpload"
        set_job_execution_status(
            config.clone(),
            job_execution.id.clone(),
            JobExecutionStatus::PendingArtifactUpload,
        )
        .await;

        // Upload output artifacts for job execution
        info!("Uplodaing output artifacts for job execution");
        prepare_output_artifacts(config.clone(), job_execution.id.clone(), job.project_id).await;

        // Update job execution status to "Completed"
        set_job_execution_status(
            config.clone(),
            job_execution.id.clone(),
            JobExecutionStatus::Completed,
        )
        .await;

        // Clean up
        fs::remove_dir_all(config.get_job_execution_path(job_execution.id.to_string().as_str()))
            .expect("could not remove job execution directory");
    }
}
