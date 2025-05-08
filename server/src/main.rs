mod thread_pool;
mod handler;

use std::env;
use std::net::TcpListener;
use handler::handle_connection;
use thread_pool::ThreadPool;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut directory = "";

    if let Some(i) = args.iter().position(|x| x == "--directory") {
        if let Some(dir) = args.get(i + 1) {
            directory = dir;
        }
    }

    let listener = TcpListener::bind("127.0.0.1:4221").expect("Could not bind to port");
    let pool = ThreadPool::new(4);

    println!("Server running on 127.0.0.1:4221");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let dir = directory.to_string();
                pool.execute(move || {
                    handle_connection(stream, &dir);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}
