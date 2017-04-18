mod request;
mod ascii;

use std::io::prelude::*;
use std::env;
use std::net::{TcpListener, TcpStream};
use std::str;

const DEFAULT_PORT: u16 = 4485;

fn generate_response(target: &str) -> &str {
    "HTTP/1.1 200 OK\r\n\r\nreplace me"
}

fn generate_error<'a>(error: request::HTTPError) -> &'a str {
    match error {
        request::HTTPError::BadRequest => "HTTP/1.1 400 Bad Request\r\n\r\n",
        request::HTTPError::NotImplemented => "HTTP/1.1 501 Not Implemented\r\n\r\n",
        request::HTTPError::VersionNotSupported => "HTTP/1.1 505 HTTP Version Not Supported\r\n\r\n",
        request::HTTPError::NotFound => "HTTP/1.1 404 Not Found\r\n\r\n",
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("read failed");

    match request::parse_request(&buffer) {
        Ok(request) => {
            let response = match request.target {
                "/" => generate_response("/"),
                _ => generate_error(request::HTTPError::NotFound),
            };
            stream.write(response.as_bytes()).expect("write failed");
        },
        Err(error) => {
            let response = generate_error(error);
            stream.write(response.as_bytes()).expect("write failed");
        },
    }
}

fn main() {
    let port: u16 = match env::var("PORT") {
        Ok(port) => port.trim().parse().expect("$PORT is not an integer"),
        Err(_) => DEFAULT_PORT
    };

    println!("Binding to port {}", port);

    let listener = TcpListener::bind(("0.0.0.0", port)).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(_) => {
                panic!("error accepting connection");
            }
        }
    }
}
