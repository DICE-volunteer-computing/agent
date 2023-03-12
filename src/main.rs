use mongodb::bson::doc;
use rust_sdk::api::{host, job, job_execution, runtime};
use rust_sdk::model::host::{Configuration, CreateHostDTO, Status as HostStatus, UpdateHostDTO};
use rust_sdk::model::job_execution::{Status as JobExecutionStatus, UpdateJobExecutionDTO};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Cursor;
use std::process::Command;
use std::{io, thread, time::Duration};

#[tokio::main]
async fn main() {
    run().await.expect("Could not run agent");
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

        // Download input artifacts to "/<Job Execution ID>/input" directory
        println!("Downloading input artifacts for job execution");

        // Run job execution
        println!("Running job execution");

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
            .expect("Could not delete runtime files");
    }
}
