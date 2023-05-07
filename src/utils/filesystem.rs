use std::{
    fs::{self, File},
    io::{self, Cursor, Read},
    process::Command,
};

pub fn get_wasm_file_in_dir(root: &str) -> String {
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

pub fn list_files_in_dir(root: &str) -> io::Result<Vec<String>> {
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

pub async fn download_file(uri: String, filename: &str) {
    let response = reqwest::get(uri).await.expect("could not download file");

    let mut created_file = File::create(&filename).expect("could not create file");
    let mut content = Cursor::new(
        response
            .bytes()
            .await
            .expect("could not get file download bytes"),
    );
    std::io::copy(&mut content, &mut created_file).expect("Could not copy data to file");
}

pub async fn upload_file(uri: String, buffer: Vec<u8>) {
    reqwest::Client::new()
        .put(uri)
        .body(buffer)
        .send()
        .await
        .expect("could not upload file");

    // TODO: Add retry logic in the event that upload fails
}

pub fn load_file_to_memory(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).expect("could not open file");

    // Read the file contents into a buffer
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Could not read tar file");

    buffer
}

pub fn untar_file(filename: &str) {
    Command::new("tar")
        .arg("-xvf")
        .arg(filename)
        .status()
        .expect("Could not untar the file");
}

pub fn tar_file(filename: &str, output: &str) {
    Command::new("tar")
        .arg("-czf")
        .arg(filename.clone())
        .arg(output)
        .status()
        .expect("Could not tar the output artifact");
}
