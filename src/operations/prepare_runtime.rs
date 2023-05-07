use std::{env::set_current_dir, fs};

use mongodb::bson::oid::ObjectId;
use rust_sdk::api::runtime;

use crate::{
    config::config::AgentConfig,
    utils::filesystem::{download_file, untar_file},
};

pub async fn prepare_runtime(config: AgentConfig<'_>, job_execution_id: ObjectId, id: ObjectId) {
    fs::create_dir_all(config.get_runtime_path(
        job_execution_id.to_string().as_str(),
        id.to_string().as_str(),
    ))
    .expect("could not create directory");

    set_current_dir(config.get_runtime_path(
        job_execution_id.to_string().as_str(),
        id.to_string().as_str(),
    ))
    .expect("could not set directory");

    let runtime_tar_file = format!("{}.tar", id);
    let runtime_download_response = runtime::download(config.sdk_config.clone(), id).await;

    download_file(runtime_download_response.uri, runtime_tar_file.as_str()).await;
    untar_file(runtime_tar_file.as_str());

    fs::remove_file(runtime_tar_file.as_str()).expect("could not remove file");

    set_current_dir(config.root).expect("could not set current directory");
}
