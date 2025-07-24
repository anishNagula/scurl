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
    headers: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let mut request_builder = match method {
        "POST" => {
            if let Some(data) = body {
                client.post(url).header(USER_AGENT, "scurl/0.1").body(data.to_string())
            } else {
                client.post(url).header(USER_AGENT, "scurl/0.1")
            }
        }
        _ => client.get(url).header(USER_AGENT, "scurl/0.1"),
    };

    for header in headers {
        if let Some((key, value)) = header.split_once(':') {
            request_builder = request_builder.header(key.trim(), value.trim());
        }
    }

    let mut response = request_builder.send()?;
    if !response.status().is_success() {
        return Err(format!("Request failed with status: {}", response.status()).into());
    }

    if let Some(file_path) = output {
        save_to_file(&mut response, file_path)?;
    } else {
        let text = response.text()?;
        println!("{}", text);
    }

    Ok(())
}

fn save_to_file(
    response: &mut reqwest::blocking::Response,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
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

    let mut downloaded = 0u64;
    let mut buffer = [0; 65536]; // 64 KB buffer
    while let Ok(read) = response.read(&mut buffer) {
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        downloaded += read as u64;

        if total_size > 0 {
            print!("\rDownloading: {}/{} bytes", downloaded, total_size);
        } else {
            print!("\rDownloading: {} bytes", downloaded);
        }
        std::io::stdout().flush().unwrap();
    }

    println!("\nSaved to {}", file_path);
    Ok(())
}
