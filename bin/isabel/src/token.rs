pub type Token = [u8; 16];

pub fn parse_token(hex: &str) -> Token {
    assert_eq!(hex.len(), 32);

    let mut result = [0; 16];
    let mut idx = 0;
    let iterator = TokenIterator { token: hex, pos: 0 };

    for value in iterator {
        result[idx] = value;
        idx += 1;
    }

    result
}

struct TokenIterator<'t> {
    token: &'t str,
    pos: usize,
}

impl TokenIterator<'_> {
    fn next_value(&mut self) -> Option<u8> {
        if self.pos >= self.token.len() {
            return None;
        }

        let value = self.token.as_bytes()[self.pos];
        self.pos += 1;

        match value {
            b'0'..=b'9' => Some(value - b'0'),
            b'a'..=b'f' => Some(value - b'a' + 10),
            _ => None,
        }
    }
}

impl Iterator for TokenIterator<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let v1 = self.next_value()?;
        let v2 = self.next_value()?;

        Some((v1 << 4) + v2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_next_value() {
        let mut iter = TokenIterator {
            token: "9876543210fedcba",
            pos: 0,
        };

        assert_eq!(iter.next_value(), Some(9));
        assert_eq!(iter.next_value(), Some(8));
        assert_eq!(iter.next_value(), Some(7));
        assert_eq!(iter.next_value(), Some(6));
        assert_eq!(iter.next_value(), Some(5));
        assert_eq!(iter.next_value(), Some(4));
        assert_eq!(iter.next_value(), Some(3));
        assert_eq!(iter.next_value(), Some(2));
        assert_eq!(iter.next_value(), Some(1));
        assert_eq!(iter.next_value(), Some(0));

        assert_eq!(iter.next_value(), Some(0xf));
        assert_eq!(iter.next_value(), Some(0xe));
        assert_eq!(iter.next_value(), Some(0xd));
        assert_eq!(iter.next_value(), Some(0xc));
        assert_eq!(iter.next_value(), Some(0xb));
        assert_eq!(iter.next_value(), Some(0xa));

        assert_eq!(iter.next_value(), None);
    }

    #[test]
    fn test_iterator() {
        let mut iter = TokenIterator {
            token: "9876543210fedcba",
            pos: 0,
        };

        assert_eq!(iter.next(), Some(0x98));
        assert_eq!(iter.next(), Some(0x76));
        assert_eq!(iter.next(), Some(0x54));
        assert_eq!(iter.next(), Some(0x32));
        assert_eq!(iter.next(), Some(0x10));
        assert_eq!(iter.next(), Some(0xfe));
        assert_eq!(iter.next(), Some(0xdc));
        assert_eq!(iter.next(), Some(0xba));
    }

    #[test]
    fn test_parse() {
        let token = "59565144447659713237774434425a7a";
        assert_eq!(parse_token(token), hex!("59565144447659713237774434425a7a"));
    }
}
