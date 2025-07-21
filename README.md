# SCurl  
**Simpler, barebones curl clone written in Rust**

---

## Features (so far)
- **GET and POST requests**  
- **Custom headers** with `-H "Key: Value"`  
- **File download** with progress bar (`-o <file>`)  
- **Basic request body support** for POST (`-d <data>`)  
- **Logs each step** of the request process  
- **Text vs binary output detection**

---

## Installation
Clone the repo and build using Cargo:
```bash
git clone https://github.com/<your-username>/scurl.git
cd scurl
cargo build --release
```
The compiled binary will be available at:
```bash
target/release/scurl
```

## Usage
### GET Request
```bash
cargo run -- get https://example.com
```
### GET Request with File Download
```bash
cargo run -- get https://example.com -o page.html
```

### POST Request with Data
```bash
cargo run -- post https://httpbin.org/post -d '{"name":"anish"}'
```
### Custom Headers
```bash
cargo run -- get https://httpbin.org/headers -H "Accept: application/json" -H "User-Agent: SCurl"
```
### POST with Data and Headers
```bash
cargo run -- post https://httpbin.org/post -d '{"name":"anish"}' -H "Content-Type: application
```

## Planned Features
- File uploads (-F file=@path)
- Timeout and retry options
- Verbose/quiet modes
- Streaming and piping support
- Automated tests

## Project Status
This is an experimental, work-in-progress CLI tool. It currently implements GET, POST, and file download features with a lightweight design.