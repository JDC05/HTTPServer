# Rust TCP File Server

A lightweight multithreaded TCP server written in Rust. Supports file uploading (POST), downloading (GET), echo responses (with optional gzip encoding), and user-agent inspection.

Example Requests
Echo:
curl http://127.0.0.1:4221/echo/hello

Gzip Echo:
curl -H "Accept-Encoding: gzip" http://127.0.0.1:4221/echo/hello --output - | gunzip

User-Agent:
curl http://127.0.0.1:4221/user-agent

Upload file:
curl -X POST --data-binary @file.txt http://127.0.0.1:4221/files/file.txt

Download file:
curl http://127.0.0.1:4221/files/file.txt


Project Structure
server/
├── src/
│   ├── main.rs         # Entry point
│   ├── thread_pool.rs  # Simple thread pool implementation
│   └── handler.rs      # Handles HTTP request logic
├── Cargo.toml
└── .gitignore


Todo:
Add HTTPS support
Graceful shutdown
Logging
MIME type detection
