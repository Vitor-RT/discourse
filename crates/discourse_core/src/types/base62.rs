const KEYS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const BASE: u64 = KEYS.len() as u64;

pub fn encode(mut num: u64) -> String {
    if num == 0 {
        return "0".to_string();
    }

    let mut result = Vec::new();
    while num > 0 {
        result.push(KEYS[(num % BASE) as usize]);
        num /= BASE;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

pub fn decode(str: &str) -> u64 {
    let mut num = 0;
    for c in str.chars() {
<<<<<<< HEAD
        let val = if c.is_ascii_digit() {
            c as u64 - '0' as u64
        } else if c.is_ascii_lowercase() {
            c as u64 - 'a' as u64 + 10
        } else if c.is_ascii_uppercase() {
=======
        let val = if ('0'..='9').contains(&c) {
            c as u64 - '0' as u64
        } else if ('a'..='z').contains(&c) {
            c as u64 - 'a' as u64 + 10
        } else if ('A'..='Z').contains(&c) {
>>>>>>> origin/rust-migration-6812869939882800025
            c as u64 - 'A' as u64 + 36
        } else {
            0
        };
        num = num * BASE + val;
    }
    num
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(encode(0), "0");
        assert_eq!(encode(1), "1");
        assert_eq!(encode(61), "Z");
        assert_eq!(encode(62), "10");
        assert_eq!(encode(63), "11");
        assert_eq!(encode(999), "g7");
    }

    #[test]
    fn test_decode() {
        assert_eq!(decode("0"), 0);
        assert_eq!(decode("1"), 1);
        assert_eq!(decode("Z"), 61);
        assert_eq!(decode("10"), 62);
        assert_eq!(decode("11"), 63);
        assert_eq!(decode("g7"), 999);
    }
}
