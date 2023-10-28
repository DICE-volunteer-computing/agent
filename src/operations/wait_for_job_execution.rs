use std::{thread, time::Duration};

use log::info;
use mongodb::bson::{doc, oid::ObjectId};
use rust_sdk::{
    api::job_execution,
    model::job_execution::{JobExecution, JobExecutionStatus},
    utils::conversion::convert_enum_to_string,
};

use crate::config::config::AgentConfig;

async fn list_pending_job_executions(config: AgentConfig, id: ObjectId) -> Vec<JobExecution> {
    let my_doc = doc! {
        "status": convert_enum_to_string(JobExecutionStatus::PendingExecution),
        "host_id": id,
    };
    info!("{:?}", my_doc);

    job_execution::list(
        config.sdk_config.clone(),
        doc! {
            "status": convert_enum_to_string(JobExecutionStatus::PendingExecution),
            "host_id": id,
        },
    )
    .await
}

pub async fn wait_for_job_execution(config: AgentConfig, id: ObjectId) -> JobExecution {
    let mut job_executions = vec![];

    while job_executions.len() == 0 {
        job_executions = list_pending_job_executions(config.clone(), id.clone()).await;

        info!("Waiting for job execution...");
        thread::sleep(Duration::from_secs(config.work_check_interval_seconds));
    }

    // Get the first job execution from the list of available job executions
    job_executions.remove(0)
}
