use hyper::{Body, Client, Request};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::body::HttpBody as _;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::sync::Arc;
use std::time::Instant;

use crate::utils::progress_bar;

/// maximum no.of redirects allowed
const MAX_REDIRECTS: usize = 5;

// create reusable static client for connection pooling
static CLIENT: once_cell::sync::Lazy<Arc<Client<HttpsConnector<HttpConnector>>>> = once_cell::sync::Lazy::new(|| {
    let https = HttpsConnector::new();
    let client = Client::builder()
        .pool_max_idle_per_host(10)
        .http1_title_case_headers(true)
        .build::<_, Body>(https);
    Arc::new(client)
});


pub async fn perform_request(
    method: &str,
    url: &str,
    body: Option<&str>,
    output: Option<&str>,
    headers: &[String],
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = CLIENT.clone();
    let mut current_url = url.to_string();

    let start = Instant::now();

    for redirect_count in 0..=MAX_REDIRECTS {
        if verbose {
            eprintln!("> {} {}", method, current_url);
            for h in headers {
                eprintln!("> {}", h);
            }
            if let Some(data) = body {
                eprintln!("> Body: {} bytes", data.len());
            }
        }

        let mut req_builder = Request::builder()
            .method(method)
            .uri(&current_url)
            .header("User-Agent", "scurl/0.2");

        // Insert custom headers
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

        let mut response = client.request(request).await?;

        if verbose {
            eprintln!("< HTTP/{:?} {}", response.version(), response.status());
            for (key, value) in response.headers() {
                eprintln!("< {}: {}", key, value.to_str().unwrap_or("<invalid>"));
            }
        }

        // follow redirects upto MAX_REDIRECTS
        if response.status().is_redirection() {
            if let Some(location) = response.headers().get("Location") {
                current_url = location.to_str()?.to_string();
                eprintln!("Redirecting to: {}", current_url);
                if redirect_count == MAX_REDIRECTS {
                    return Err("Too many redirects".into());
                }
                continue;
            }
        }

        if !response.status().is_success() {
            return Err(format!("Request failed: {}", response.status()).into());
        }

        // for HEAD
        if method.eq_ignore_ascii_case("HEAD") {
            if !verbose {
                eprintln!("HTTP/{:?} {}", response.version(), response.status());
                for (key, value) in response.headers() {
                    eprintln!("{}: {}", key, value.to_str().unwrap_or("<invalid>"));
                }
            }
            eprintln!("Time taken: {:.2?}", start.elapsed());
            return Ok(());
        }

        // for GET/POST
        if let Some(path) = output {
            save_to_file(&mut response, path, verbose).await?;
        } else {
            print_to_stdout(&mut response).await?;
        }

        eprintln!("Time taken: {:.2?}", start.elapsed());
        return Ok(());
    }

    Err("Failed after redirects".into())
}


// save response body to file with buffered writes and progress bar
async fn save_to_file(
    response: &mut hyper::Response<Body>,
    path: &str,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let total_size = response
        .headers()
        .get("Content-Length")
        .and_then(|val| val.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    // skip progress bar for small/unknown size files (<50KB)
    let use_progress = total_size > 50_000;

    let mut pb = if use_progress {
        Some(progress_bar(total_size))
    } else {
        None
    };

    let mut downloaded: u64 = 0;

    while let Some(chunk) = response.body_mut().data().await {
        let data = chunk?;
        writer.write_all(&data)?;
        downloaded += data.len() as u64;

        if let Some(pb) = pb.as_mut() {
            pb.update(downloaded);
        }
    }

    writer.flush()?;

    if let Some(mut pb) = pb {
        pb.finish();
    }

    if verbose {
        eprintln!("Saved to {}", path);
    }
    Ok(())
}

/// print response body directly to stdout
async fn print_to_stdout(
    response: &mut hyper::Response<Body>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stdout = io::stdout();
    while let Some(chunk) = response.body_mut().data().await {
        stdout.write_all(&chunk?)?;
    }
    Ok(())
}
