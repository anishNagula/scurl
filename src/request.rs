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
    let client = Client::builder()
        .build()?;

    let request = match method {
        "POST" => {
            if let Some(data) = body {
                client.post(url).header(USER_AGENT, "scurl/0.1").body(data.to_string())
            } else {
                client.post(url).header(USER_AGENT, "scurl/0.1")
            }
        }
        _ => client.get(url).header(USER_AGENT, "scurl/0.1"),
    };

    let mut response = request.send()?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("Request failed with status: {}", status).into());
    }

    if let Some(path) = output {
        save_to_file(&mut response, path)?;
    } else {
        print_text_response(&mut response)?;
    }

    Ok(())
}

fn save_to_file(response: &mut reqwest::blocking::Response, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(path);
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
    let mut buffer = [0u8; 131072]; // 64 KB buffer

    while let Ok(n) = response.read(&mut buffer) {
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        downloaded += n as u64;
    }

    if total_size > 0 && downloaded != total_size {
        eprintln!("Warning: downloaded {} bytes, expected {}", downloaded, total_size);
    }

    Ok(())
}

fn print_text_response(response: &mut reqwest::blocking::Response) -> Result<(), Box<dyn std::error::Error>> {
    let mut body = String::new();
    response.read_to_string(&mut body)?;
    println!("{body}");
    Ok(())
}
