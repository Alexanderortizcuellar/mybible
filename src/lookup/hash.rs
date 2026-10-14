/// Computes Java's `java.lang.String.hashCode()` on a UTF-8 string.
///
/// In Java: `s[0]*31^(n-1) + s[1]*31^(n-2) + ... + s[n-1]`
/// where calculations are performed on 16-bit UTF-16 code units using 32-bit signed integer arithmetic.
pub fn java_hash_code(s: &str) -> i32 {
    let mut h: i32 = 0;
    for code_unit in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(code_unit as i32);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_hash_code() {
        // Known Java hashCodes:
        // "" -> 0
        // "a" -> 97
        // "hello" -> 99162322
        // "G1" -> 70 * 31 + 49 = 2170 + 49 = 2219
        assert_eq!(java_hash_code(""), 0);
        assert_eq!(java_hash_code("a"), 97);
        assert_eq!(java_hash_code("hello"), 99162322);
        assert_eq!(java_hash_code("G1"), 2250);
    }
}
