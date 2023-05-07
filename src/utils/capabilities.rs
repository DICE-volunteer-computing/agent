use std::process::Command;

use rust_sdk::model::common::{PlatformArchitecture, PlatformExecutionType};

pub fn get_device_architecture() -> PlatformArchitecture {
    let output = String::from_utf8_lossy(
        &Command::new("uname")
            .arg("-m")
            .output()
            .expect("could not run uname command")
            .stdout,
    )
    .into_owned();
    let platform = output.strip_suffix("\n").expect("could not strip newline");

    match platform {
        "arm64" => PlatformArchitecture::Arm64,
        "aarch64" => PlatformArchitecture::Arm64,
        "x86_64" => PlatformArchitecture::X86_64,
        _ => panic!("could not identify PlatformArchitecture"),
    }
}

pub fn check_wasmer() -> bool {
    Command::new("wasmer")
        .arg("--version")
        .status()
        .expect("could not check wasmer")
        .success()
}

pub fn check_docker() -> bool {
    Command::new("docker")
        .arg("--version")
        .status()
        .expect("could not check wasmer")
        .success()
}

pub fn get_platform_architectures() -> Vec<PlatformArchitecture> {
    let mut platform_architectures = vec![];
    platform_architectures.insert(0, get_device_architecture());

    if check_wasmer() {
        platform_architectures.insert(0, PlatformArchitecture::Wasm);
    }

    platform_architectures
}

pub fn get_platform_execution_types() -> Vec<PlatformExecutionType> {
    let mut platform_execution_types = vec![PlatformExecutionType::BareMetal];

    if check_wasmer() {
        platform_execution_types.insert(0, PlatformExecutionType::Wasmer);
    }

    if check_docker() {
        platform_execution_types.insert(0, PlatformExecutionType::Docker);
    }

    platform_execution_types
}
