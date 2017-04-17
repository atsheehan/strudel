pub type Ascii = [u8];

const LINE_FEED: u8 = 10;
const CARRIAGE_RETURN: u8 = 13;
const SPACE: u8 = 32;

pub fn read_line(buffer: &Ascii) -> Option<(&Ascii, &Ascii)> {
    let mut cr_found = false;

    for (index, byte) in buffer.iter().enumerate() {
        if *byte == CARRIAGE_RETURN {
            cr_found = true;
        } else if cr_found && *byte == LINE_FEED {
            return Some((&buffer[..(index - 1)], &buffer[(index + 1)..]));
        } else {
            cr_found = false;
        }
    }

    None
}

pub fn read_token(buffer: &Ascii) -> (&Ascii, &Ascii) {
    for (index, byte) in buffer.iter().enumerate() {
        if *byte == SPACE {
            return (&buffer[..index], &buffer[(index + 1)..]);
        }
    }

    (buffer, b"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_a_line_ending_with_crlf() {
        let input = b"this is the first line\r\n";

        let (line, remaining) = read_line(input).unwrap();

        assert_eq!(line, b"this is the first line");
        assert_eq!(remaining, b"");
    }

    #[test]
    fn it_reads_multiple_lines() {
        let input = b"first line\r\nsecond line\r\n";

        let (first_line, input) = read_line(input).unwrap();
        let (second_line, _) = read_line(input).unwrap();

        assert_eq!(first_line, b"first line");
        assert_eq!(second_line, b"second line");
    }

    #[test]
    fn it_interprets_double_crlf_as_empty_string() {
        let input = b"two crlf\r\n\r\n";

        let (first_line, remaining) = read_line(input).unwrap();
        let (second_line, _) = read_line(remaining).unwrap();

        assert_eq!(first_line, b"two crlf");
        assert_eq!(second_line, b"");
    }

    #[test]
    fn it_fails_when_no_crlf_found() {
        let input = b"there is no crlf here";
        let result = read_line(input);
        assert!(result.is_none());
    }

    #[test]
    fn it_reads_consecutive_tokens_separated_by_spaces() {
        let input = b"first second third";

        let (first, input) = read_token(input);
        let (second, input) = read_token(input);
        let (third, _) = read_token(input);

        assert_eq!(first, b"first");
        assert_eq!(second, b"second");
        assert_eq!(third, b"third");
    }

    #[test]
    fn it_returns_the_input_when_no_spaces() {
        let input = b"nospaceshere";

        let (token, remaining) = read_token(input);

        assert_eq!(token, b"nospaceshere");
        assert_eq!(remaining, b"");
    }
}
