mod config;
mod operations;
mod threads;
mod utils;

use log::info;
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
    let config = AgentConfig::dev_default();
    fs::create_dir_all(config.root).expect("could not create root directory");
    set_current_dir(config.root).expect("could not set current directory");

    // --- REGISTER HOST ---
    let host_id = register_host(config.clone()).await;
    info!("Host ID: {}", host_id);

    // --- HEARTBEAT ---
    // TODO: Split out heartbeat thread

    // --- MAIN LOOP ---
    job_processor(config.clone(), host_id.clone()).await;
}
