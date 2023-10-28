mod config;
mod operations;
mod threads;
mod utils;

use log::info;
use rust_sdk::utils::env::get_registration_token;
use simple_logger::SimpleLogger;
use std::env::set_current_dir;
use std::fs;

use crate::config::config::AgentConfig;
use crate::operations::register::register_host;
use crate::threads::job_processor::job_processor;

#[tokio::main]
async fn main() {
    // --- SETUP RUNTIME DEPENDENCIES ---
    SimpleLogger::new()
        .init()
        .expect("could not create SimpleLogger");

    info!("--- DICE Agent ---");

    // --- SPECIFY CONFIGURATION ---
    let mut config = AgentConfig::dev_default(get_registration_token());
    fs::create_dir_all(config.clone().root).expect("could not create root directory");
    set_current_dir(config.clone().root).expect("could not set current directory");

    // --- REGISTER HOST ---
    let create_host_response = register_host(config.clone()).await;
    config = AgentConfig::dev_default(create_host_response.token);
    info!("Host ID: {}", create_host_response.id);

    // --- HEARTBEAT ---
    // TODO: Split out heartbeat thread

    // --- MAIN LOOP ---
    job_processor(config, create_host_response.id).await;
}
