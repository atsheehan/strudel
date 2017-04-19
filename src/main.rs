mod request;
mod ascii;

use std::io::prelude::*;
use std::env;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::fs::File;

const DEFAULT_PORT: u16 = 4485;

fn generate_response<'a, 'b>(target: &'a str, routes: &'b HashMap<&str, Vec<u8>>) -> &'b [u8] {
    match routes.get(target) {
        Some(content) => content.as_slice(),
        None => generate_error(request::HTTPError::NotFound),
    }
}

fn generate_error<'a>(error: request::HTTPError) -> &'a [u8] {
    match error {
        request::HTTPError::BadRequest => b"HTTP/1.1 400 Bad Request\r\n\r\n",
        request::HTTPError::NotImplemented => b"HTTP/1.1 501 Not Implemented\r\n\r\n",
        request::HTTPError::VersionNotSupported => b"HTTP/1.1 505 HTTP Version Not Supported\r\n\r\n",
        request::HTTPError::NotFound => b"HTTP/1.1 404 Not Found\r\n\r\n",
    }
}

fn handle_client(mut stream: TcpStream, routes: &HashMap<&str, Vec<u8>>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("read failed");

    match request::parse_request(&buffer) {
        Ok(request) => {
            let response = generate_response(request.target, routes);
            stream.write(response).expect("write failed");
        },
        Err(error) => {
            let response = generate_error(error);
            stream.write(response).expect("write failed");
        },
    }
}

fn read_file(path: &str) -> Vec<u8> {
    let mut file = File::open(path).expect(&format!("Could not open file at {}", path));
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect(&format!("Failed to read {}", path));
    buffer
}

fn main() {
    let port: u16 = match env::var("PORT") {
        Ok(port) => port.trim().parse().expect("$PORT is not an integer"),
        Err(_) => DEFAULT_PORT
    };

    println!("Binding to port {}", port);

    let mut routes: HashMap<&str, Vec<u8>> = HashMap::new();
    routes.insert("/", read_file("templates/home.html"));

    let listener = TcpListener::bind(("0.0.0.0", port)).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream, &routes);
            }
            Err(_) => {
                panic!("error accepting connection");
            }
        }
    }
}
