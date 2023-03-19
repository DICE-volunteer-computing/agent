use mongodb::bson::doc;
use rust_sdk::api::{host, job, job_execution, runtime};
use rust_sdk::model::artifact::{
    ArtifactType, CreateArtifactDTO, Status as ArtifactStatus, UpdateArtifactDTO,
};
use rust_sdk::model::entity::EntityType;
use rust_sdk::model::host::{Configuration, CreateHostDTO, Status as HostStatus, UpdateHostDTO};
use rust_sdk::model::job_execution::{Status as JobExecutionStatus, UpdateJobExecutionDTO};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Cursor, Read};
use std::process::Command;
use std::{thread, time::Duration};

#[tokio::main]
async fn main() {
    run().await.expect("Could not run agent");
}

fn get_wasm_file_in_dir(root: &str) -> String {
    let read_result = fs::read_dir(root).expect("Could not read dir");
    for path in read_result {
        let path = path.expect("Could not resolve path").path();
        let str_path = path.to_str().expect("No string path");

        if str_path.clone().contains(".wasm") {
            return format!("{}", str_path);
        }
    }

    panic!("No wasm file in directory")
}

fn list_files_in_dir(root: &str) -> io::Result<Vec<String>> {
    let mut result = vec![];

    for path in fs::read_dir(root)? {
        result.push(
            path?
                .path()
                .to_owned()
                .to_str()
                .expect("Could not get string of path")
                .to_string(),
        );
    }

    Ok(result)
}

async fn run() -> Result<(), io::Error> {
    // --- SETUP RUNTIME DEPENDENCIES ---
    println!("Setting up dependencies");

    // Config
    let root_path = "/tmp";
    let runtime_path = "dice.runtime";
    env::set_current_dir(root_path)?;

    // Agent setup
    //  1. Load registration token
    //  [DONE] 2. Register host
    //  3. Load tags from env vars, update host tags
    //  [DONE] 4. Create temporary working directory in /tmp

    // --- REGISTER HOST ---
    println!("Registering host with DICE");
    let host_id = host::create(CreateHostDTO {
        user_id: format!("6403b8a4001c963eebd9c4fd"),
        tags: HashMap::new(),
        configuration: Configuration {
            mem_bytes: 1233453,
            disk_bytes: 123434534,
            cores: 12,
        },
    })
    .await;
    println!("Host ID: {}", host_id.clone());

    // --- MAIN LOOP ---
    loop {
        // Update host status to Idle
        println!("Updating host status to \"Idle\"");
        host::update(
            host_id.clone(),
            UpdateHostDTO {
                status: HostStatus::Idle,
            },
        )
        .await;

        // Query for job executions
        println!("Waiting for job execution...");
        let mut job_executions = job_execution::list(doc! {
            "status": serde_json::to_string(&JobExecutionStatus::PendingExecution).unwrap().replace("\"", ""),
            "host_id": { "$oid": host_id.clone() },
        })
        .await;
        while job_executions.len() == 0 {
            thread::sleep(Duration::from_secs(1));

            println!("Waiting for job execution...");
            job_executions = job_execution::list(doc! {
                "status": serde_json::to_string(&JobExecutionStatus::PendingExecution).unwrap().replace("\"", ""),
                "host_id": { "$oid": host_id.clone() },
            })
            .await;
        }

        // Get the first job execution from the list of available job executions
        let job_execution = job_executions.get(0).unwrap();
        let job = job::get(job_execution.job_id.to_string()).await;

        // Update host status to "Busy"
        println!("Updating host status to \"Busy\"");
        host::update(
            host_id.clone(),
            UpdateHostDTO {
                status: HostStatus::Busy,
            },
        )
        .await;

        // Update job execution status to "Execution"
        println!("Updating job execution status to \"Execution\"");
        job_execution::update(
            job_execution.id.clone().to_string(),
            UpdateJobExecutionDTO {
                status: JobExecutionStatus::Execution,
            },
        )
        .await;

        // Create temporary "/<Job Execution ID>/input" and "/<Job Execution ID>/output" directories
        fs::create_dir_all(runtime_path)?;

        println!("Creating temporary input and output directories for job execution");
        let input_path = format!("{}/input", &job_execution.id.to_string());
        fs::create_dir_all(format!("{}/{}", root_path, &input_path))?;
        println!("Job Execution input temporary directory: {}", &input_path);
        let output_path = format!("{}/output", &job_execution.id.to_string());
        fs::create_dir_all(format!("{}/{}", root_path, &output_path))?;
        println!("Job Execution output temporary directory: {}", &output_path);

        // Download runtime to temporary working directory
        let runtime_file_path = format!("{}/{}", runtime_path, job.runtime_id.clone().to_string());
        let runtime_tar_file_path = format!("{}.tar", runtime_file_path);
        println!("Downloading runtime for job execution");

        let runtime_download_response = runtime::download(job.runtime_id.to_string()).await;
        let response = reqwest::get(runtime_download_response.uri).await.unwrap();

        let mut runtime_file = File::create(&runtime_tar_file_path).unwrap();
        let mut content = Cursor::new(response.bytes().await.unwrap());
        std::io::copy(&mut content, &mut runtime_file).expect("Could not copy runtime to file");

        Command::new("tar")
            .arg("-xvf")
            .arg(runtime_tar_file_path.clone())
            .arg("-C")
            .arg(runtime_path)
            .status()
            .expect("Could not untar the runtime");
        let wasm_file = get_wasm_file_in_dir(runtime_path);

        // Download input artifacts to "/<Job Execution ID>/input" directory
        println!("Downloading input artifacts for job execution");

        let artifacts = rust_sdk::api::artifact::list(doc! {
            "artifact_type": serde_json::to_string(&ArtifactType::Input).unwrap().replace("\"", ""),
            "_id": { "$in": job.input_artifacts },
            "status": serde_json::to_string(&ArtifactStatus::Active).unwrap().replace("\"", "")
        })
        .await;

        env::set_current_dir(&input_path).expect("Could not change directories");

        // For each artifact in job execution, download it, untar it, and then remove the tar file
        let task_handles = artifacts.into_iter().map(|artifact| {
            tokio::spawn(async move {
                //  Download artifact
                let tar_file_path = format!("{}.tar", artifact.id.to_string());

                let download_artifact_response =
                    rust_sdk::api::artifact::download(artifact.id.to_string()).await;
                let response = reqwest::get(download_artifact_response.uri).await.unwrap();

                let mut artifact_file = File::create(&tar_file_path).unwrap();
                let mut content = Cursor::new(response.bytes().await.unwrap());
                std::io::copy(&mut content, &mut artifact_file)
                    .expect("Could not copy artifact to file");

                //  Untar the artifact
                Command::new("tar")
                    .arg("-xvf")
                    .arg(tar_file_path.clone())
                    .status()
                    .expect("Could not untar the input artifact");

                //  Delete tar file
                Command::new("rm")
                    .arg(tar_file_path)
                    .status()
                    .expect("Could not delete tar file");
            })
        });

        for handler in task_handles {
            handler.await.expect("Could not download input artifact");
        }

        env::set_current_dir("../../").expect("Could not change directories");

        // Run job execution
        println!("Running job execution");

        Command::new("wasmer")
            .arg("run")
            .arg(format!("--mapdir=input:{}", input_path))
            .arg(format!("--mapdir=output:{}", output_path))
            .arg(wasm_file)
            .status()
            .expect("Could not run runtime");

        // Update job execution status to "PendingArtifactUpload"
        println!("Updating job execution status to \"PendingArtifactUpload\"");
        job_execution::update(
            job_execution.id.clone().to_string(),
            UpdateJobExecutionDTO {
                status: JobExecutionStatus::PendingArtifactUpload,
            },
        )
        .await;

        // Upload output artifacts for job execution
        println!("Uplodaing output artifacts for job execution");

        env::set_current_dir(&output_path).expect("Could not change directories");

        // For each artifact in job execution, download it, untar it, and then remove the tar file
        let task_handles = list_files_in_dir(".")
            .expect("Could not list files in output dir")
            .into_iter()
            .map(|artifact| {
                tokio::spawn(async move {
                    // Get job execution ID
                    let path_str = env::current_dir().expect("Could not get current directory");
                    let path_components = path_str
                        .to_str()
                        .expect("Could not get string path of current directory")
                        .split("/")
                        .collect::<Vec<&str>>();
                    let job_execution_id = path_components
                        .get(path_components.len().wrapping_sub(2))
                        .expect("Could not find job execution id")
                        .to_string();

                    // Create artifact
                    let create_artifact_response =
                        rust_sdk::api::artifact::create(CreateArtifactDTO {
                            entity_id: job_execution_id,
                            entity_type: EntityType::JobExecution,
                            artifact_type: ArtifactType::Output,
                            tags: HashMap::new(),
                        })
                        .await;

                    //  Download artifact
                    let tar_file_path = format!("{}.tar", create_artifact_response.id.to_string());

                    let download_artifact_response =
                        rust_sdk::api::artifact::download(create_artifact_response.id.to_string())
                            .await;
                    let response = reqwest::get(download_artifact_response.uri).await.unwrap();

                    let mut artifact_file = File::create(&tar_file_path).unwrap();
                    let mut content = Cursor::new(response.bytes().await.unwrap());
                    std::io::copy(&mut content, &mut artifact_file)
                        .expect("Could not copy artifact to file");

                    //  Tar the artifact
                    Command::new("tar")
                        .arg("-czf")
                        .arg(tar_file_path.clone())
                        .arg(artifact)
                        .status()
                        .expect("Could not tar the output artifact");

                    // Load runtime file
                    let mut file =
                        File::open(tar_file_path.clone()).expect("Could not open tar file");

                    // Read the file contents into a buffer
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)
                        .expect("Could not read tar file");

                    // Upload the compressed file
                    let upload_response = reqwest::Client::new()
                        .put(create_artifact_response.uri)
                        .body(buffer)
                        .send()
                        .await;
                    match upload_response {
                        Ok(_) => {
                            println!("Successfully uploaded output artifact");

                            // Set input artifact status to active
                            rust_sdk::api::artifact::update(
                                create_artifact_response.id.clone(),
                                UpdateArtifactDTO {
                                    status: ArtifactStatus::Active,
                                },
                            )
                            .await;

                            println!("Created output artifact: {}", create_artifact_response.id);
                        }
                        Err(err) => println!("Could not upload output artifact: {}", err),
                    };
                })
            });

        for handler in task_handles {
            handler.await.expect("Could not upload ouput artifact");
        }

        env::set_current_dir(root_path).expect("Could not change directories");

        // Update job execution status to "Completed"
        println!("Updating job execution status to \"Completed\"");
        job_execution::update(
            job_execution.id.clone().to_string(),
            UpdateJobExecutionDTO {
                status: JobExecutionStatus::Completed,
            },
        )
        .await;

        Command::new("rm")
            .arg("-rf")
            .arg(runtime_path)
            .arg(job_execution.id.to_string())
            .status()
            .expect("Could not delete files");
    }
}
