use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn};
use reqwest::blocking::Client;
use reqwest::header::{self, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn perform_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    output: Option<&str>,
    headers: &[String],
) -> Result<()> {
    info!("Starting {} request to {}", method, url);
    let client = Client::new();

    let mut request_builder = match method {
        "POST" => {
            info!("Preparing POST request");
            let mut rb = client.post(url).header(USER_AGENT, "SCurl/0.1");
            if let Some(data) = body {
                info!("POST data: {}", data);
                rb = rb.body(data.to_string());
            }
            rb
        }
        _ => {
            info!("Preparing GET request");
            client.get(url).header(USER_AGENT, "SCurl/0.1")
        }
    };

    // custom headers
    for header in headers {
        if let Some((key, value)) = header.split_once(':') {
            info!("Adding header: {}: {}", key.trim(), value.trim());
            request_builder = request_builder.header(key.trim(), value.trim());
        } else {
            warn!("Invalid header format: {}", header);
        }
    }

    info!("Sending request...");
    let mut response = request_builder.send()?;

    let status = response.status();
    info!("Received response: {}", status);

    if !status.is_success() {
        return Err(anyhow!("Request failed with status: {}", status));
    }

    if let Some(file_path) = output {
        save_response_to_file(&mut response, file_path)?;
    } else {
        print_response(&mut response)?;
    }

    Ok(())
}


fn save_response_to_file(
    response: &mut reqwest::blocking::Response,
    file_path: &str,
) -> Result<()> {
    info!("Saving response to file: {}", file_path);

    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            info!("Creating directories: {:?}", parent);
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = File::create(path)?;

    let total_size = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    info!("Total size: {} bytes", total_size);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("🚀 [{bar:40.green/blue}] {bytes}/{total_bytes} • {bytes_per_sec} • ETA {eta}")
            .unwrap()
            .progress_chars("█░"),
    );

    let mut downloaded = 0u64;
    let mut buffer = [0; 131072];
    while let Ok(read) = response.read(&mut buffer) {
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        downloaded += read as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download complete");
    info!("File saved: {}", file_path);
    Ok(())
}

fn print_response(response: &mut reqwest::blocking::Response) -> Result<()> {
    if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
        let ct = content_type.to_str().unwrap_or("");
        if ct.starts_with("text/") || ct.contains("json") {
            let mut body = String::new();
            response.read_to_string(&mut body)?;
            println!("{}", body);
        } else {
            warn!("Response looks like binary ({}), won't print to stdout.", ct);
        }
    } else {
        let mut body = String::new();
        response.read_to_string(&mut body)?;
        println!("{}", body);
    }
    Ok(())
}
