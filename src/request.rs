use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::fs;

pub fn perform_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    output: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let request_builder = match method {
        "POST" => {
            if let Some(data) = body {
                client.post(url).header(USER_AGENT, "SCurl/0.1").body(data.to_string())
            } else {
                client.post(url).header(USER_AGENT, "SCurl/0.1")
            }
        }
        _ => client.get(url).header(USER_AGENT, "SCurl/0.1"),
    };

    let response = request_builder.send()?;
    let status = response.status();
    let content = response.bytes()?;

    if status.is_success() {
        if let Some(file_path) = output {
            let path = Path::new(file_path);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut file = File::create(path)?;
            file.write_all(&content)?;
            println!("Saved response to {}", file_path);
        } else {
            println!("{}", String::from_utf8_lossy(&content));
        }
        Ok(())
    } else {
        Err(format!("Request failed with status: {}", status).into())
    }
}
