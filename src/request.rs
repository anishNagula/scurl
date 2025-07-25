use hyper::body::HttpBody as _;
use hyper::{Body, Client, Request};
use hyper_tls::HttpsConnector;
use std::fs::File;
use std::io::{self, Write};
use std::time::Instant;

/// Perform GET or POST request
pub async fn perform_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    output: Option<&str>,
    headers: &[String],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, Body>(https);

    // Build request
    let mut req_builder = Request::builder()
        .method(method)
        .uri(url)
        .header("User-Agent", "SCurl/0.1");

    for header in headers {
        if let Some((k, v)) = header.split_once(':') {
            req_builder = req_builder.header(k.trim(), v.trim());
        }
    }

    let request = if let Some(data) = body {
        req_builder.body(Body::from(data.to_owned()))?
    } else {
        req_builder.body(Body::empty())?
    };

    let start = Instant::now();
    let mut response = client.request(request).await?;

    if !response.status().is_success() {
        return Err(format!("Request failed: {}", response.status()).into());
    }

    if let Some(path) = output {
        save_to_file(&mut response, path).await?;
    } else {
        print_to_stdout(&mut response).await?;
    }

    eprintln!("Completed in {:.2?}", start.elapsed());
    Ok(())
}

/// Save response body to a file with a minimal progress output
async fn save_to_file(
    response: &mut hyper::Response<Body>,
    path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = File::create(path)?;
    let mut total = 0usize;

    while let Some(chunk) = response.body_mut().data().await {
        let data = chunk?;
        file.write_all(&data)?;
        total += data.len();
        eprint!("\rDownloading: {} bytes", total);
    }

    eprintln!("\nSaved to {}", path);
    Ok(())
}

/// Print response body to stdout
async fn print_to_stdout(
    response: &mut hyper::Response<Body>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stdout = io::stdout();
    while let Some(chunk) = response.body_mut().data().await {
        stdout.write_all(&chunk?)?;
    }
    Ok(())
}