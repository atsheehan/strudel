use std::collections::HashMap;
use std::str;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum HTTPError {
    BadRequest,
    NotImplemented,
    VersionNotSupported,
    NotFound,
}

#[derive(Debug)]
pub struct RequestLine<'a> {
    pub method: &'a str,
    pub target: &'a str,
    pub http_version: &'a str,
}

#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a str,
    pub target: &'a str,
    pub http_version: &'a str,
    pub headers: HashMap<&'a str, &'a str>,
}

pub fn parse_request(buffer: &[u8]) -> Result<Request, HTTPError> {
    match read_header_line(buffer) {
        Some((line, buffer)) => {
            let request_line = match parse_request_line(line) {
                Some(request_line) => request_line,
                None => return Err(HTTPError::BadRequest),
            };

            match validate_request_line(request_line) {
                Ok(request_line) => {
                    match parse_request_headers(buffer) {
                        Some(headers) => {
                            let request = Request {
                                method: request_line.method,
                                target: request_line.target,
                                http_version: request_line.http_version,
                                headers: headers,
                            };

                            Ok(request)
                        },
                        None => Err(HTTPError::BadRequest),
                    }
                },
                Err(error) => Err(error),
            }
        },
        None => Err(HTTPError::BadRequest)
    }
}

fn parse_request_line(line: &str) -> Option<RequestLine> {
    let tokens: Vec<&str> = line.split(' ').collect();

    if tokens.len() == 3 {
        Some(RequestLine { method: tokens[0], target: tokens[1], http_version: tokens[2] })
    } else {
        None
    }
}

fn parse_request_headers(mut buffer: &[u8]) -> Option<HashMap<&str, &str>> {
    let mut headers = HashMap::new();

    loop {
        match read_header_line(buffer) {
            Some((line, remaining)) => {
                buffer = remaining;

                if line.is_empty() {
                    return Some(headers);
                }

                let tokens: Vec<&str> = line.split(':').collect();
                if tokens.len() != 2 {
                    return None;
                }

                let key = tokens[0];
                let value = tokens[1].trim();

                headers.insert(key, value);
            },
            None => return None,
        };
    }
}

fn validate_request_line(request: RequestLine) -> Result<RequestLine, HTTPError> {
    if request.http_version != "HTTP/1.1" {
        return Err(HTTPError::VersionNotSupported);
    }

    if request.method != "GET" {
        return Err(HTTPError::NotImplemented);
    }

    Ok(request)
}

const LINE_FEED: u8 = 10;
const CARRIAGE_RETURN: u8 = 13;

/// Returns the next line from a request header. The line must be
/// terminated by a CRLF pair according to Section 3 of RFC 7230. If
/// the header does not contain a CRLF or the line is not
/// ASCII-encoded, returns None.
fn read_header_line<'a>(header: &'a [u8]) -> Option<(&'a str, &'a [u8])> {
    let mut cr_found = false;

    for (index, byte) in header.iter().enumerate() {
        if *byte == CARRIAGE_RETURN {
            cr_found = true;
        } else if cr_found && *byte == LINE_FEED {
            let line = &header[..(index - 1)];
            let remaining = &header[(index + 1)..];

            return match str::from_utf8(line) {
                Ok(line) => Some((line, remaining)),
                Err(_) => None,
            }
        } else {
            cr_found = false;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request_with_no_headers() {
        let input = b"GET /foo HTTP/1.1\r\n\r\n";
        let request = parse_request(input).unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.target, "/foo");
        assert_eq!(request.http_version, "HTTP/1.1");
    }

    #[test]
    fn parse_request_with_headers() {
        let input = b"GET /foo HTTP/1.1\r\nHost: example.com\r\nAccept: text/html\r\n\r\n";
        let request = parse_request(input).unwrap();

        assert_eq!(*request.headers.get("Host").unwrap(), "example.com");
        assert_eq!(*request.headers.get("Accept").unwrap(), "text/html");
    }

    #[test]
    fn parse_request_returns_400_if_request_line_has_too_many_words() {
        let input = b"GET /foo HTTP/1.1 bar\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::BadRequest);
    }

    #[test]
    fn parse_request_returns_400_if_request_line_has_too_few_words() {
        let input = b"GET /foo\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::BadRequest);
    }

    #[test]
    fn parse_request_returns_400_if_request_line_has_more_than_single_space() {
        let input = b"GET  /foo HTTP/1.1\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::BadRequest);
    }

    #[test]
    fn parse_request_returns_400_if_request_line_uses_other_whitespace() {
        let input = b"GET\t/foo HTTP/1.1\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::BadRequest);
    }

    #[test]
    fn parse_request_returns_501_for_unrecognized_methods() {
        let input = b"PROPFIND /foo HTTP/1.1\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::NotImplemented);
    }

    #[test]
    fn parse_request_returns_505_unless_using_http_1_1() {
        let input = b"GET /foo HTTP/1.0\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::VersionNotSupported);
    }

    #[test]
    fn parse_request_returns_400_if_request_line_contains_non_ascii_chars() {
        let input = b"GET /foo\xFF HTTP/1.0\r\n\r\n";
        let error = parse_request(input).unwrap_err();

        assert_eq!(error, HTTPError::BadRequest);
    }

    #[test]
    fn read_header_line_reads_consecutive_lines_split_by_crlf() {
        let buffer = b"first line\r\nsecond line\r\n";

        let (first_line, buffer) = read_header_line(buffer).unwrap();
        let (second_line, _) = read_header_line(buffer).unwrap();

        assert_eq!(first_line, "first line");
        assert_eq!(second_line, "second line");
    }

    #[test]
    fn read_header_line_returns_none_if_missing_crlf() {
        let buffer = b"no crlf";
        let result = read_header_line(buffer);
        assert!(result.is_none());
    }

    #[test]
    fn read_header_line_returns_none_if_line_isnt_ascii() {
        let buffer = b"non-ascii \xFF char\r\n";
        let result = read_header_line(buffer);
        assert!(result.is_none());
    }
}
