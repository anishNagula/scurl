use hyper::{Body, Client, Request};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use hyper::body::HttpBody as _;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::sync::Arc;
use std::time::Instant;

use crate::utils::progress_bar;

/// Maximum no.of redirects allowed
const MAX_REDIRECTS: usize = 5;

/// Number of parallel chunks to fetch (can be tuned)
const PARALLEL_CHUNKS: u64 = 32;

// Create reusable static client for connection pooling
static CLIENT: once_cell::sync::Lazy<Arc<Client<HttpsConnector<HttpConnector>>>> =
    once_cell::sync::Lazy::new(|| {
        let https = HttpsConnector::new();
        let client = Client::builder()
            .pool_max_idle_per_host(50)
            .http1_title_case_headers(true)
            .build::<_, Body>(https);
        Arc::new(client)
    });

async fn print_to_stdout(
    response: &mut hyper::Response<Body>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stdout = io::stdout();
    while let Some(chunk) = response.body_mut().data().await {
        stdout.write_all(&chunk?)?;
    }
    Ok(())
}

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

        // Follow redirects up to MAX_REDIRECTS
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

        // For HEAD
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

        // For GET/POST
        if let Some(path) = output {
            save_to_file(&mut response, path, url, headers, verbose).await?;
        } else {
            print_to_stdout(&mut response).await?;
        }

        eprintln!("Time taken: {:.2?}", start.elapsed());
        return Ok(());
    }

    Err("Failed after redirects".into())
}

async fn save_to_file(
    response: &mut hyper::Response<Body>,
    path: &str,
    url: &str,
    headers: &[String],
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let total_size = response
        .headers()
        .get("Content-Length")
        .and_then(|val| val.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    if total_size > 5_000_000 && total_size > 0 {
        if verbose {
            eprintln!("Using parallel download for {} ({} bytes)", path, total_size);
        }
        return parallel_download(url, path, total_size, headers, verbose).await;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let pb = if total_size > 50_000 && total_size > 0 {
        Some(progress_bar(total_size))
    } else {
        None
    };

    while let Some(chunk) = response.body_mut().data().await {
        let data = chunk?;
        writer.write_all(&data)?;

        if let Some(ref bar) = pb {
            bar.inc(data.len() as u64);
        }
    }

    writer.flush()?;

    if let Some(bar) = pb {
        bar.finish();
    }

    if verbose {
        eprintln!("Saved to {}", path);
    }

    Ok(())
}



/// Parallel chunked download
async fn parallel_download(
    url: &str,
    path: &str,
    total_size: u64,
    headers: &[String],
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chunk_size = total_size / PARALLEL_CHUNKS;
    let pb = progress_bar(total_size);

    let mut tasks = vec![];
    for i in 0..PARALLEL_CHUNKS {
        let start = i * chunk_size;
        let end = if i == PARALLEL_CHUNKS - 1 {
            total_size - 1
        } else {
            (i + 1) * chunk_size - 1
        };

        let range = format!("bytes={}-{}", start, end);
        let url = url.to_string();
        let headers = headers.to_vec();
        let pb_clone = pb.clone();

        tasks.push(tokio::spawn(async move {
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, Body>(https);

            let mut req_builder = Request::builder()
                .method("GET")
                .uri(&url)
                .header("User-Agent", "scurl/0.2")
                .header("Range", range);

            for header in &headers {
                if let Some((k, v)) = header.split_once(':') {
                    req_builder = req_builder.header(k.trim(), v.trim());
                }
            }

            let mut res = client.request(req_builder.body(Body::empty())?).await?;
            let mut data = Vec::new();

            while let Some(chunk) = res.body_mut().data().await {
                let c = chunk?;
                pb_clone.inc(c.len() as u64);
                data.extend_from_slice(&c);
            }

            Ok::<(u64, Vec<u8>), Box<dyn std::error::Error + Send + Sync>>((start, data))
        }));
    }

    let mut file = OpenOptions::new().create(true).write(true).open(path)?;
    for task in tasks {
        let (start, data) = task.await??;
        file.seek(SeekFrom::Start(start))?;
        file.write_all(&data)?;
    }

    pb.finish();
    if verbose {
        eprintln!("Saved to {}", path);
    }
    Ok(())
}