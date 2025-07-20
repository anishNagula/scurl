use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;

pub fn perform_request(
    method: &str,
    url: &str,
    body: Option<&str>,
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
    if response.status().is_success() {
        let text = response.text()?;
        println!("{}", text);
        Ok(())
    } else {
        Err(format!("Request failed with status: {}", response.status()).into())
    }
}
