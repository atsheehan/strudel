mod base64;
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
    println!("\nis_websocket: {}\n\n", request.is_websocket());

    if request.is_websocket() {
        connect_websocket(request, stream);
    } else {
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
}

const BONUS_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

fn connect_websocket(request: request::Request, mut stream: TcpStream) {
    if !request.headers.get("sec-websocket-version").map_or(false, |version| *version == "13") {
        write_error(request::HTTPError::BadRequest, stream);
        return;
    }

    let websocket_key = match request.headers.get("sec-websocket-key") {
        Some(key) => key,
        None => {
            write_error(request::HTTPError::BadRequest, stream);
            return;
        },
    };

    let websocket_key = websocket_key.to_string() + BONUS_STRING;

    println!("websocket key: {}", websocket_key);

    let mut digester = sha1::SHA1Context::new();
    digester.add(&websocket_key.as_bytes());
    let websocket_key_bytes = digester.digest();

    let encoded_websocket_key = base64::encode(&websocket_key_bytes);

    let mut output_buffer: Vec<u8> = Vec::new();

    output_buffer.extend_from_slice(b"HTTP/1.1 101 Switching Protocols\r\n");
    output_buffer.extend_from_slice(b"Upgrade: websocket\r\n");
    output_buffer.extend_from_slice(b"Connection: Upgrade\r\n");
    output_buffer.extend_from_slice(b"Sec-WebSocket-Accept: ");
    output_buffer.extend_from_slice(&encoded_websocket_key);
    output_buffer.extend_from_slice(b"\r\n\r\n");

    stream.write(&output_buffer).expect("oh no");
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
