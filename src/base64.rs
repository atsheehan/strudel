fn encode_byte(input: u8) -> u8 {
    match input {
        0 => b'A',
        1 => b'B',
        2 => b'C',
        3 => b'D',
        4 => b'E',
        5 => b'F',
        6 => b'G',
        7 => b'H',
        8 => b'I',
        9 => b'J',
        10 => b'K',
        11 => b'L',
        12 => b'M',
        13 => b'N',
        14 => b'O',
        15 => b'P',
        16 => b'Q',
        17 => b'R',
        18 => b'S',
        19 => b'T',
        20 => b'U',
        21 => b'V',
        22 => b'W',
        23 => b'X',
        24 => b'Y',
        25 => b'Z',
        26 => b'a',
        27 => b'b',
        28 => b'c',
        29 => b'd',
        30 => b'e',
        31 => b'f',
        32 => b'g',
        33 => b'h',
        34 => b'i',
        35 => b'j',
        36 => b'k',
        37 => b'l',
        38 => b'm',
        39 => b'n',
        40 => b'o',
        41 => b'p',
        42 => b'q',
        43 => b'r',
        44 => b's',
        45 => b't',
        46 => b'u',
        47 => b'v',
        48 => b'w',
        49 => b'x',
        50 => b'y',
        51 => b'z',
        52 => b'0',
        53 => b'1',
        54 => b'2',
        55 => b'3',
        56 => b'4',
        57 => b'5',
        58 => b'6',
        59 => b'7',
        60 => b'8',
        61 => b'9',
        62 => b'+',
        63 => b'/',
        _ => panic!("Invalid Base64 value: {}", input),
    }
}

pub fn encode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();

    for chunk in input.chunks(3) {
        let mut word: u32 = 0;

        for (index, byte) in chunk.iter().enumerate() {
            let bitshift = 24 - (index * 8);
            word |= (*byte as u32) << bitshift;
        }

        output.push(encode_byte((word >> 26) as u8 & 0b00111111));
        output.push(encode_byte((word >> 20) as u8 & 0b00111111));

        match chunk.len() {
            1 => {
                output.push(b'=');
                output.push(b'=');
            },
            2 => {
                output.push(encode_byte((word >> 14) as u8 & 0b00111111));
                output.push(b'=');
            },
            3 => {
                output.push(encode_byte((word >> 14) as u8 & 0b00111111));
                output.push(encode_byte((word >> 8) as u8 & 0b00111111));
            },
            _ => panic!(),
        };

    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_passes_test_cases_without_padding_from_rfc4648() {
        assert_eq!(&encode(b""), b"");
        assert_eq!(&encode(b"foo"), b"Zm9v");
        assert_eq!(&encode(b"foobar"), b"Zm9vYmFy");
    }

    #[test]
    fn encode_passes_test_cases_with_padding_from_rfc4648() {
        assert_eq!(&encode(b"f"), b"Zg==");
        assert_eq!(&encode(b"fo"), b"Zm8=");
        assert_eq!(&encode(b"foob"), b"Zm9vYg==");
        assert_eq!(&encode(b"fooba"), b"Zm9vYmE=");
    }
}
