use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use flate2::write::GzEncoder;
use flate2::Compression;

pub fn handle_connection(mut stream: TcpStream, directory: &str) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(n) if n == 0 => return,
        Ok(n) => {
            let request = String::from_utf8_lossy(&buffer[..n]);
            let request_line = request.lines().next().unwrap_or("");
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            let method = parts.get(0).unwrap_or(&"");
            let path = parts.get(1).unwrap_or(&"/");
            let content_length = request
                .lines()
                .find(|l| l.to_lowercase().starts_with("content-length:"))
                .and_then(|l| l.split(':').nth(1))
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(0);

            let wants_close = request.lines().any(|l| l.to_lowercase().contains("connection: close"));
            let connection_header = if wants_close { "Connection: close\r\n" } else { "" };

            let header_end = request.find("\r\n\r\n").unwrap_or(n);
            let body = &buffer[header_end + 4..header_end + 4 + content_length.min(buffer.len().saturating_sub(header_end + 4))];

            let response = match (*method, *path) {
                ("GET", "/") => format!("HTTP/1.1 200 OK\r\n{}\r\n", connection_header),
                ("GET", p) if p.starts_with("/echo/") => {
                    let value = &p[6..];
                    if request.to_lowercase().contains("accept-encoding: gzip") {
                        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                        encoder.write_all(value.as_bytes()).unwrap();
                        let compressed = encoder.finish().unwrap();
                        let mut resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\n{}\r\n",
                            compressed.len(), connection_header
                        ).into_bytes();
                        resp.extend(compressed);
                        stream.write_all(&resp).unwrap();
                        return;
                    } else {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n{}\r\n{}",
                            value.len(), connection_header, value
                        )
                    }
                }
                ("GET", "/user-agent") => {
                    let ua = request
                        .lines()
                        .find(|l| l.starts_with("User-Agent:"))
                        .unwrap_or("User-Agent: ")
                        .trim_start_matches("User-Agent: ")
                        .trim();
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n{}\r\n{}",
                        ua.len(), connection_header, ua
                    )
                }
                ("GET", p) if p.starts_with("/files/") => {
                    let filename = &p[7..];
                    let path = format!("{}/{}", directory, filename);
                    match fs::read(&path) {
                        Ok(contents) => {
                            let mut response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n{}\r\n",
                                contents.len(), connection_header
                            ).into_bytes();
                            response.extend(contents);
                            stream.write_all(&response).unwrap();
                            return;
                        }
                        Err(_) => format!("HTTP/1.1 404 Not Found\r\n{}\r\n", connection_header),
                    }
                }
                ("POST", p) if p.starts_with("/files/") => {
                    let filename = &p[7..];
                    let path = format!("{}/{}", directory, filename);
                    match fs::write(&path, body) {
                        Ok(_) => format!("HTTP/1.1 201 Created\r\n{}\r\n", connection_header),
                        Err(_) => format!("HTTP/1.1 500 Internal Server Error\r\n{}\r\n", connection_header),
                    }
                }
                _ => format!("HTTP/1.1 404 Not Found\r\n{}\r\n", connection_header),
            };

            stream.write_all(response.as_bytes()).unwrap();
        }
        Err(e) => eprintln!("Failed to read request: {}", e),
    }
}
