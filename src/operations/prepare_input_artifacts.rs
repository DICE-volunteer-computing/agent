use std::{env::set_current_dir, fs};

use mongodb::bson::oid::ObjectId;
use rust_sdk::api::artifact;

use crate::{
    config::config::AgentConfig,
    utils::filesystem::{download_file, untar_file},
};

pub async fn prepare_input_artifacts(
    config: AgentConfig<'_>,
    id: ObjectId,
    input_artifacts: Vec<ObjectId>,
) {
    set_current_dir(config.get_job_execution_input_path(id.to_string().as_str()))
        .expect("could not set current directory");

    for artifact_id in input_artifacts {
        prepare_input_artifact(config.clone(), artifact_id).await
    }

    set_current_dir(config.root).expect("could not set current directory");
}

async fn prepare_input_artifact(config: AgentConfig<'_>, id: ObjectId) {
    let artifact_tar_file = format!("{}.tar", id);
    let artifact_download_response = artifact::download(config.sdk_config, id).await;

    download_file(artifact_download_response.uri, artifact_tar_file.as_str()).await;
    untar_file(artifact_tar_file.as_str());

    fs::remove_file(artifact_tar_file.as_str()).expect("could not remove file");
}
