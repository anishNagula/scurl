use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Result, anyhow};

pub fn perform_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    output: Option<&str>,
    headers: &[String],
) -> Result<()> {
    let client = Client::new();

    let mut request_builder = match method {
        "POST" => {
            let mut rb = client.post(url).header(USER_AGENT, "scurl/0.1");
            if let Some(data) = body {
                rb = rb.body(data.to_string());
            }
            rb
        }
        "PUT" => {
            let mut rb = client.put(url).header(USER_AGENT, "scurl/0.1");
            if let Some(data) = body {
                rb = rb.body(data.to_string());
            }
            rb
        }
        "DELETE" => client.delete(url).header(USER_AGENT, "scurl/0.1"),
        _ => client.get(url).header(USER_AGENT, "scurl/0.1"),
    };

    for header in headers {
        if let Some((key, value)) = header.split_once(':') {
            request_builder = request_builder.header(key.trim(), value.trim());
        } else {
            eprintln!("Warning: Invalid header format '{}', expected 'Key: Value'", header);
        }
    }

    let mut response = request_builder.send()?;
    let status = response.status();

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
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = File::create(path)?;
    let mut buffer = [0; 8192];
    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 { break; }
        file.write_all(&buffer[..read])?;
    }

    println!("Saved to {}", file_path);
    Ok(())
}

fn print_response(response: &mut reqwest::blocking::Response) -> Result<()> {
    let mut body = String::new();
    response.read_to_string(&mut body)?;
    println!("{}", body);
    Ok(())
}
