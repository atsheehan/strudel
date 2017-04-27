mod request;
mod sha1;

use std::io::prelude::*;
use std::env;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::fs::File;

const DEFAULT_PORT: u16 = 4485;

fn write_response(request: request::Request, mut stream: TcpStream, routes: &HashMap<&str, String>) {
    println!("REQUEST -- {} {} {}", request.method, request.target, request.http_version);
    for (header, value) in &request.headers {
        println!("{}: \"{}\"", header, value);
    }
    println!();

    match routes.get(request.target) {
        Some(content) => {
            let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/html;charset=utf-8\r\n\r\n";
            let mut response = String::with_capacity(headers.len() + content.len());

            response.push_str(headers);
            response.push_str(&content);
            stream.write(response.as_bytes()).expect("failed to write response");
        },
        None => write_error(request::HTTPError::NotFound, stream),
    }
}

fn write_error(error: request::HTTPError, mut stream: TcpStream) {
    let contents: &[u8] = match error {
        request::HTTPError::BadRequest => b"HTTP/1.1 400 Bad Request\r\n\r\nBad Request",
        request::HTTPError::NotImplemented => b"HTTP/1.1 501 Not Implemented\r\n\r\nNot Implemented",
        request::HTTPError::VersionNotSupported => b"HTTP/1.1 505 HTTP Version Not Supported\r\n\r\nHTTP Version Not Supported",
        request::HTTPError::NotFound => b"HTTP/1.1 404 Not Found\r\n\r\nNot Found",
    };

    stream.write(contents).expect("failed to write response");
}

fn handle_client(mut stream: TcpStream, routes: &HashMap<&str, String>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("read failed");

    match request::parse_request(&buffer) {
        Ok(request) => write_response(request, stream, routes),
        Err(error) => write_error(error, stream),
    };
}

fn read_file(path: &str) -> String {
    let mut file = File::open(path).expect(&format!("Could not open file at {}", path));
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect(&format!("Failed to read {}", path));
    contents
}

fn main() {
    let port: u16 = match env::var("PORT") {
        Ok(port) => port.trim().parse().expect("$PORT is not an integer"),
        Err(_) => DEFAULT_PORT
    };

    let mut routes: HashMap<&str, String> = HashMap::new();
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
