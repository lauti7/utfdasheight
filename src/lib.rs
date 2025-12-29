pub fn utf8_encode(codepoint: i32) -> Vec<u8> {
    let one_byte_range = 0x0..=0x7f;
    let two_bytes_range = 0x80..=0x7ff;
    let three_bytes_range = 0x800..=0xffff;
    let four_bytes_range = 0x10000..=0x10ffff;

    if one_byte_range.contains(&codepoint) {
        let octet = 0b00000000_u8;
        let in_u8 = codepoint as u8;

        // ascii codepoint. codepoint can be directly returned instead of masking
        return vec![octet | in_u8];
    }

    let continuation_octet = 0b1000_0000_u8;
    // create this mask to turn on the bits that are turned on `codepoint`
    let continuation_mask = 0b0011_1111_u8;

    if two_bytes_range.contains(&codepoint) {
        let continuation_result = continuation_octet | (continuation_mask & codepoint as u8);

        let leading_octet = 0b1100_0000_u8;
        // same here with the leading octet
        let leading_mask = 0b0011_1111_u8;
        // we do a right shift 6 bits because these bits are the ones used on the continuation octet (0b10xx_xxxx_u8)
        let leading_result = leading_octet | (leading_mask & (codepoint as u8 >> 6));

        return vec![leading_result, continuation_result];
    }

    if three_bytes_range.contains(&codepoint) {
        let last_continuation_octet = continuation_octet | (continuation_mask & codepoint as u8);
        let first_continuation_octet =
            continuation_octet | (continuation_mask & (codepoint >> 6) as u8);

        let leading_octet = 0b1110_0000_u8;
        let leading_mask = 0b0001_1111_u8;
        let leading_result = leading_octet | (leading_mask & (codepoint >> 12) as u8);

        return vec![
            leading_result,
            first_continuation_octet,
            last_continuation_octet,
        ];
    }

    if four_bytes_range.contains(&codepoint) {
        let last_continuation_octet = continuation_octet | (continuation_mask & codepoint as u8);
        let second_continuation_octect =
            continuation_octet | (continuation_mask & (codepoint >> 6) as u8);
        let first_continuation_octet =
            continuation_octet | (continuation_mask & (codepoint >> 12) as u8);

        let leading_octet = 0b1111_0000_u8;
        let leading_mask = 0b0000_1111_u8;
        let leading_result = leading_octet | (leading_mask & (codepoint >> 18) as u8);

        return vec![
            leading_result,
            first_continuation_octet,
            second_continuation_octect,
            last_continuation_octet,
        ];
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_encode_two_bytes_range_test() {
        let res = utf8_encode(0xA1);
        assert_eq!(res[0], 0b11000010_u8);
        assert_eq!(res[1], 0b10100001_u8);
    }

    #[test]
    fn utf8_encode_three_bytes_range_test() {
        let res = utf8_encode(0x0801);
        assert_eq!(res[0], 0b11100000_u8);
        assert_eq!(res[1], 0b10100000_u8);
        assert_eq!(res[2], 0b10000001_u8);
    }

    #[test]
    fn utf8_encode_four_bytes_range_test() {
        let res = utf8_encode(0x10001);
        assert_eq!(res[0], 0b11110000_u8);
        assert_eq!(res[1], 0b10010000_u8);
        assert_eq!(res[2], 0b10000000_u8);
        assert_eq!(res[3], 0b10000001_u8);
    }

    #[test]
    fn utf8_encode_no_range() {
        let res = utf8_encode(0x11ffff);
        assert!(res.is_empty())
    }
}
