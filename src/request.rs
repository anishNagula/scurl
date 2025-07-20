use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::header::{USER_AGENT, CONTENT_LENGTH};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

pub fn perform_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    output: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting {} request to {}", method, url);

    let client = Client::new();

    let request_builder = match method {
        "POST" => {
            println!("Preparing POST request");
            if let Some(data) = body {
                client.post(url).header(USER_AGENT, "SCurl/0.1").body(data.to_string())
            } else {
                client.post(url).header(USER_AGENT, "SCurl/0.1")
            }
        }
        _ => {
            println!("Preparing GET request");
            client.get(url).header(USER_AGENT, "SCurl/0.1")
        }
    };

    println!("Sending request...");
    let mut response = request_builder.send()?;

    let status = response.status();
    println!("Received response: {}", status);

    if !status.is_success() {
        return Err(format!("Request failed with status: {}", status).into());
    }

    if let Some(file_path) = output {
        println!("Saving response to file: {}", file_path);
        let path = Path::new(file_path);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                println!("Creating directories: {:?}", parent);
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

        println!("Total size to download: {} bytes", total_size);

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
            .template("🚀 [{bar:40.green/blue}] {bytes}/{total_bytes} • {bytes_per_sec} • ETA {eta}")
            .unwrap()
            .progress_chars("█░"),
        );

        let mut downloaded = 0u64;
        let mut buffer = [0; 8192];
        while let Ok(read) = response.read(&mut buffer) {
            if read == 0 {
                break;
            }

            file.write_all(&buffer[..read])?;
            downloaded += read as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");
        println!("Saved response to: {}", file_path);
    } else {
        println!("No output file specified, printing response:");
        let content = response.text()?;
        println!("{}", content);
    }

    Ok(())
}
