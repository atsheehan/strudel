use std::str;
use std::ascii::AsciiExt;
use ascii;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum HTTPError {
    BadRequest,
    NotImplemented,
    VersionNotSupported,
}

#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a str,
    pub target: &'a str,
    pub http_version: &'a str,
}

pub fn parse_request(buffer: &ascii::Ascii) -> Result<Request, HTTPError> {
    match ascii::read_line(buffer) {
        Some((line, _buffer)) => {
            if !line.is_ascii() {
                return Err(HTTPError::BadRequest);
            }

            let (method, line) = ascii::read_token(line);
            let (target, line) = ascii::read_token(line);
            let (http_version, _) = ascii::read_token(line);

            let method = str::from_utf8(method).unwrap();
            let target = str::from_utf8(target).unwrap();
            let http_version = str::from_utf8(http_version).unwrap();

            let request = Request { method: method, target: target, http_version: http_version };
            validate_request(request)
        },
        None => Err(HTTPError::BadRequest)
    }
}

fn validate_request(request: Request) -> Result<Request, HTTPError> {
    if request.method != "GET" {
        Err(HTTPError::NotImplemented)
    } else {
        if request.http_version != "HTTP/1.1" {
            Err(HTTPError::VersionNotSupported)
        } else {
            Ok(request)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_a_request_with_no_headers() {
        let input = b"GET /foo HTTP/1.1\r\n\r\n";
        let request = parse_request(input).unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.target, "/foo");
        assert_eq!(request.http_version, "HTTP/1.1");
    }

    #[test]
    fn it_returns_501_for_unrecognized_methods() {
        let input = b"PROPFIND /foo HTTP/1.1\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::NotImplemented);
    }

    #[test]
    fn it_returns_505_unless_using_http_1_1() {
        let input = b"GET /foo HTTP/1.0\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::VersionNotSupported);
    }

    #[test]
    fn it_returns_400_if_request_line_contains_non_ascii_chars() {
        let input = b"GET /foo\xFF HTTP/1.0\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::BadRequest);
    }
}
