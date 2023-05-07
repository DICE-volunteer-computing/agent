use std::{collections::HashMap, env::set_current_dir};

use mongodb::bson::oid::ObjectId;
use rust_sdk::{
    api::artifact,
    model::artifact::{ArtifactStatus, CreateArtifactDTO},
};

use crate::{
    config::config::AgentConfig,
    utils::filesystem::{list_files_in_dir, load_file_to_memory, tar_file, upload_file},
};

use super::update_status::set_artifact_status;

pub async fn prepare_output_artifacts(config: AgentConfig<'_>, id: ObjectId, project_id: ObjectId) {
    set_current_dir(config.get_job_execution_output_path(id.to_string().as_str()))
        .expect("could not set current directory");

    for output_artifact in list_files_in_dir(".").expect("could not list files") {
        // Create artifact
        let create_artifact_response = artifact::create(
            config.sdk_config.clone(),
            CreateArtifactDTO {
                project_id: project_id,
                tags: HashMap::new(),
            },
        )
        .await;

        //  Tar the artifact
        let tar_file_path = format!("{}.tar", create_artifact_response.id.to_string());
        tar_file(tar_file_path.as_str(), output_artifact.as_str());

        // Set input artifact status to active
        set_artifact_status(
            config.clone(),
            create_artifact_response.id,
            ArtifactStatus::Active,
        )
        .await;

        // Upload the compressed file
        upload_file(
            create_artifact_response.uri,
            load_file_to_memory(tar_file_path.as_str()),
        )
        .await;
    }

    set_current_dir(config.root).expect("could not set current directory");
}
