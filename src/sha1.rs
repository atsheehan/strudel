struct SHA1Context {
    input_buffer: [u8; 64],
    length: u64,
}

impl SHA1Context {
    fn new() -> SHA1Context {
        SHA1Context { input_buffer: [0; 64], length: 0, }
    }

    fn add(&mut self, message: &[u8]) {
        self.length = message.len() as u64;

        for (index, byte) in message.iter().enumerate() {
            self.input_buffer[index] = *byte;
        }
    }

    fn digest(&mut self) -> [u8; 20] {
        self.input_buffer[self.length as usize] = 0x80;

        let mut h: [u32; 5] = [
            0x67452301,
            0xEFCDAB89,
            0x98BADCFE,
            0x10325476,
            0xC3D2E1F0,
        ];

        if self.length > 55 {
            h = process_message_block(self.input_buffer, h);
            self.input_buffer = [0; 64];
        }

        for (index, byte) in self.length_bytes().iter().enumerate() {
            self.input_buffer[56 + index] = *byte;
        }

        h = process_message_block(self.input_buffer, h);

        [
            (h[0] >> 24) as u8,
            (h[0] >> 16) as u8,
            (h[0] >> 8) as u8,
            h[0] as u8,
            (h[1] >> 24) as u8,
            (h[1] >> 16) as u8,
            (h[1] >> 8) as u8,
            h[1] as u8,
            (h[2] >> 24) as u8,
            (h[2] >> 16) as u8,
            (h[2] >> 8) as u8,
            h[2] as u8,
            (h[3] >> 24) as u8,
            (h[3] >> 16) as u8,
            (h[3] >> 8) as u8,
            h[3] as u8,
            (h[4] >> 24) as u8,
            (h[4] >> 16) as u8,
            (h[4] >> 8) as u8,
            h[4] as u8,
        ]
    }

    fn length_bytes(&self) -> [u8; 8] {
        let length_in_bits = self.length * 8;

        [(length_in_bits >> 56) as u8,
         (length_in_bits >> 48) as u8,
         (length_in_bits >> 40) as u8,
         (length_in_bits >> 32) as u8,
         (length_in_bits >> 24) as u8,
         (length_in_bits >> 16) as u8,
         (length_in_bits >> 8) as u8,
         length_in_bits as u8]
    }
}

fn bytes_to_word(bytes: [u8; 4]) -> u32 {
    ((bytes[0] as u32) << 24) +
        ((bytes[1] as u32) << 16) +
        ((bytes[2] as u32) << 8) +
        bytes[3] as u32
}

fn process_message_block(input: [u8; 64], mut h: [u32; 5]) -> [u32; 5] {
    let mut temp: u32;
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let mut d: u32;
    let mut e: u32;

    let mut w: [u32; 80] = [0; 80];
    let mut word_buffer: [u8; 4] = [0; 4];

    for (index, chunk) in input.chunks(4).enumerate() {
        for (index, byte) in chunk.iter().enumerate() {
            word_buffer[index] = *byte;
        }

        w[index] = bytes_to_word(word_buffer);
    }

    for i in 16..80 {
        w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
    }

    a = h[0];
    b = h[1];
    c = h[2];
    d = h[3];
    e = h[4];

    for t in 0..80 {
        temp = a.rotate_left(5).wrapping_add(f(t, b, c, d)).wrapping_add(e).wrapping_add(w[t]).wrapping_add(k(t));
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = temp;
    }

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);

    h
}

fn f(t: usize, b: u32, c: u32, d: u32) -> u32 {
    if t < 20 {
        (b & c) | (!b & d)
    } else if t < 40 {
        b ^ c ^ d
    } else if t < 60 {
        (b & c) | (b & d) | (c & d)
    } else {
        b ^ c ^ d
    }
}

fn k(t: usize) -> u32 {
    if t < 20 {
        0x5A827999
    } else if t < 40 {
        0x6ED9EBA1
    } else if t < 60 {
        0x8F1BBCDC
    } else {
        0xCA62C1D6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_digest_computes_digest() {
        let input = b"abc";

        let expected = [
            0xA9, 0x99, 0x3E, 0x36, 0x47, 0x06, 0x81, 0x6A, 0xBA, 0x3E,
            0x25, 0x71, 0x78, 0x50, 0xC2, 0x6C, 0x9C, 0xD0, 0xD8, 0x9D
        ];

        let mut context = SHA1Context::new();
        context.add(input);

        assert_eq!(context.digest(), expected);
    }

    #[test]
    fn sha1_digest_processes_multiple_512_bit_blocks() {
        // This input is 448 bits, and when adding the 65 bits of
        // padding for a total of 513 bits will require two blocks to
        // be processed.
        let input = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

        let expected = [
            0x84, 0x98, 0x3E, 0x44, 0x1C, 0x3B, 0xD2, 0x6E, 0xBA, 0xAE,
            0x4A, 0xA1, 0xF9, 0x51, 0x29, 0xE5, 0xE5, 0x46, 0x70, 0xF1,
        ];

        let mut context = SHA1Context::new();
        context.add(input);

        assert_eq!(context.digest(), expected);
    }

    #[test]
    fn bytes_to_word_properly_converts() {
        assert_eq!(bytes_to_word([0, 0, 0, 0]), 0);
        assert_eq!(bytes_to_word([0, 0, 0, 0xff]), 255);
        assert_eq!(bytes_to_word([0xff, 0, 0, 0]), 4278190080);
        assert_eq!(bytes_to_word([0, 0xab, 0xcd, 0xef]), 11259375);
    }
}
