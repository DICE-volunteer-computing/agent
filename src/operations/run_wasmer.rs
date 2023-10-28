use std::process::Command;

use mongodb::bson::oid::ObjectId;

use crate::{config::config::AgentConfig, utils::filesystem::get_wasm_file_in_dir};

pub fn run_wasmer(config: AgentConfig, id: ObjectId, runtime_id: ObjectId) {
    let wasm_file = get_wasm_file_in_dir(
        &config.get_runtime_path(id.to_string().as_str(), runtime_id.to_string().as_str()),
    );

    Command::new("wasmer")
        .arg("run")
        .arg(format!(
            "--mapdir=input:{}",
            config.get_job_execution_input_path(id.to_string().as_str())
        ))
        .arg(format!(
            "--mapdir=output:{}",
            config.get_job_execution_output_path(id.to_string().as_str())
        ))
        .arg(wasm_file)
        .status()
        .expect("could not execute WASM runtime with Wasmer");
}
