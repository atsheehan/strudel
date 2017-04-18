mod request;
mod ascii;

use std::io::prelude::*;
use std::env;
use std::net::{TcpListener, TcpStream};
use std::str;

const DEFAULT_PORT: u16 = 4485;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("read failed");

    let mut response = String::with_capacity(1024);

    match request::parse_request(&buffer) {
        Ok(request) => {
            response.push_str("HTTP/1.1 200 OK\r\n\r\n");
            let response = response + "method: " + request.method + " target: " + request.target;
            stream.write(response.as_bytes()).expect("write failed");
        },
        Err(error) => {
            match error {
                request::HTTPError::BadRequest => response.push_str("HTTP/1.1 400 Bad Request\r\n\r\n"),
                request::HTTPError::NotImplemented => response.push_str("HTTP/1.1 501 Not Implemented\r\n\r\n"),
                request::HTTPError::VersionNotSupported => response.push_str("HTTP/1.1 505 HTTP Version Not Supported\r\n\r\n"),
            }
            stream.write(response.as_bytes()).expect("write failed");
        },
    }
}

fn main() {
    let port: u16 = match env::var("PORT") {
        Ok(port) => port.trim().parse().expect("$PORT is not an integer"),
        Err(_) => DEFAULT_PORT
    };

    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();

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
