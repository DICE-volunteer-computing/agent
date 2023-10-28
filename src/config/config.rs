use rust_sdk::config::config::{SdkConfig, Stage};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentConfig {
    pub root: String,
    pub sdk_config: SdkConfig,
    pub work_check_interval_seconds: u64,
}

impl AgentConfig {
    pub fn dev_default(token: String) -> Self {
        AgentConfig {
            root: format!("/tmp/dice"),
            sdk_config: SdkConfig {
                stage: Stage::Dev,
                auth: token,
            },
            work_check_interval_seconds: 1,
        }
    }

    pub fn get_job_execution_path(&self, id: &str) -> String {
        format!("{}/{}", self.root, id)
    }

    pub fn get_job_execution_input_path(&self, id: &str) -> String {
        format!("{}/input", self.get_job_execution_path(id))
    }

    pub fn get_job_execution_output_path(&self, id: &str) -> String {
        format!("{}/output", self.get_job_execution_path(id))
    }

    pub fn get_runtime_path(&self, job_execution_id: &str, id: &str) -> String {
        format!("{}/{}", self.get_job_execution_path(job_execution_id), id)
    }
}
