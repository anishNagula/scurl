use reqwest::blocking::{Client, Response};
use reqwest::header::USER_AGENT;
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use anyhow::{Result, anyhow};

pub fn perform_request(
    client: &Client,
    method: &str,
    url: &str,
    body: Option<&str>,
    output: Option<&str>,
    headers: &[String],
) -> Result<()> {
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
    response: &mut Response,
    file_path: &str,
) -> Result<()> {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(128 * 1024, file);
    let mut buffer = [0; 128 * 1024];
    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 { break; }
        writer.write_all(&buffer[..read])?;
    }

    writer.flush()?;
    println!("Saved to {}", file_path);
    Ok(())
}

fn print_response(response: &mut Response) -> Result<()> {
    let mut body = Vec::new();
    response.read_to_end(&mut body)?;
    println!("{}", String::from_utf8_lossy(&body));
    Ok(())
}
