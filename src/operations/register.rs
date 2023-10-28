use std::collections::HashMap;

use log::info;
use rust_sdk::{
    api::host,
    model::host::{Configuration, CreateHostDTO, CreateHostResponse},
};
use sysinfo::{DiskExt, System, SystemExt};

use crate::{
    config::config::AgentConfig,
    utils::capabilities::{get_platform_architectures, get_platform_execution_types},
};

pub async fn register_host(config: AgentConfig) -> CreateHostResponse {
    info!("Registering host");

    let mut sys = System::new_all();
    sys.refresh_all();
    let total_disk_space = sys
        .disks()
        .into_iter()
        .map(|disk| disk.available_space())
        .sum();

    let create_host_response = host::create(
        config.sdk_config,
        CreateHostDTO {
            tags: HashMap::new(),
            configuration: Configuration {
                mem_bytes: sys.total_memory(),
                disk_bytes: total_disk_space,
                platform_architecture_types: get_platform_architectures(),
                platform_execution_types: get_platform_execution_types(),
            },
        },
    )
    .await;

    create_host_response
}
