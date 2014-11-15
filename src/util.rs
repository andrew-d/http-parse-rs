#[inline(always)]
fn is_digit(ch: u8) -> bool {
    ch >= b'0' && ch <= b'9'
}

#[inline(always)]
fn lowercase(ch: u8) -> u8 {
    ch | 0x20
}

#[inline(always)]
fn is_alpha(ch: u8) -> bool {
    lowercase(ch) >= b'a' && lowercase(ch) <= b'z'
}

#[inline(always)]
fn is_alphanum(ch: u8) -> bool {
    is_alpha(ch) || is_digit(ch)
}

#[inline(always)]
fn is_hex(ch: u8) -> bool {
    is_digit(ch) || (lowercase(ch) >= b'a' && lowercase(ch) <= b'f')
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_is_digit() {
        for ch in b"0123456789".iter() {
            assert!(super::is_digit(*ch));
        }
        for ch in b"/:abcdefABCDEF".iter() {
            assert!(!super::is_digit(*ch));
        }
    }

    #[test]
    fn test_lowercase() {
        assert_eq!(super::lowercase(b'A'), b'a');
    }

    #[test]
    fn test_is_alpha() {
        for ch in b"abcdefghijklmnopqrstuvwxyz".iter() {
            assert!(super::is_alpha(*ch));
        }
        for ch in b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".iter() {
            assert!(super::is_alpha(*ch));
        }
        for ch in b"0123456789".iter() {
            assert!(!super::is_alpha(*ch));
        }
    }

    #[test]
    fn test_is_alphanum() {
        for ch in b"abcdef012345".iter() {
            assert!(super::is_alphanum(*ch));
        }
    }

    #[test]
    fn test_is_hex() {
        for ch in b"0123456789abcdef".iter() {
            assert!(super::is_hex(*ch));
        }
        for ch in b"ghiquvxyz".iter() {
            assert!(!super::is_hex(*ch));
        }
    }
}
