/// UTF-16 surrogates halves
const PROHIBITED_RANGE: std::ops::RangeInclusive<u32> = 0xd800..=0xdfff;
/// ASCII
const ONE_OCTET_RANGE: std::ops::RangeInclusive<u32> = 0x0..=0x7f;
const TWO_OCTETS_RANGE: std::ops::RangeInclusive<u32> = 0x80..=0x7ff;
const THREE_OCTETS_RANGE: std::ops::RangeInclusive<u32> = 0x800..=0xffff;
const FOUR_OCTETS_RANGE: std::ops::RangeInclusive<u32> = 0x10000..=0x10ffff;

const REPLACEMENT_CHAR_0X: u32 = 0xFFFD;
const REPLACEMENT_CHAR_OCTETS: [u8; 3] = [0b11101111, 0b10111111, 0b10111101]; // 0xFFFD

/// "lossy" because it doens't fail when an invalid codepoint is passed in. It returns 0xFFFD
pub fn utf8_encode_lossy(codepoint: u32) -> Vec<u8> {
    if PROHIBITED_RANGE.contains(&codepoint) {
        return REPLACEMENT_CHAR_OCTETS.to_vec();
    }

    if ONE_OCTET_RANGE.contains(&codepoint) {
        // ascii codepoint
        return vec![codepoint as u8];
    }

    let continuation_octet = 0b1000_0000_u8;
    let continuation_mask = 0b0011_1111_u8;

    if TWO_OCTETS_RANGE.contains(&codepoint) {
        let continuation_result = continuation_octet | (continuation_mask & codepoint as u8);

        let leading_octet = 0b1100_0000_u8;
        let leading_mask = !leading_octet;
        // we do a right shift 6 bits because these bits are the ones used on the continuation octet (0b10xx_xxxx_u8)
        let leading_result = leading_octet | (leading_mask & (codepoint as u8 >> 6));

        return vec![leading_result, continuation_result];
    }

    if THREE_OCTETS_RANGE.contains(&codepoint) {
        let last_continuation_octet = continuation_octet | (continuation_mask & codepoint as u8);
        let first_continuation_octet =
            continuation_octet | (continuation_mask & (codepoint >> 6) as u8);

        let leading_octet = 0b1110_0000_u8;
        let leading_mask = !leading_octet;
        let leading_result = leading_octet | (leading_mask & (codepoint >> 12) as u8);

        return vec![
            leading_result,
            first_continuation_octet,
            last_continuation_octet,
        ];
    }

    if FOUR_OCTETS_RANGE.contains(&codepoint) {
        let last_continuation_octet = continuation_octet | (continuation_mask & codepoint as u8);
        let second_continuation_octect =
            continuation_octet | (continuation_mask & (codepoint >> 6) as u8);
        let first_continuation_octet =
            continuation_octet | (continuation_mask & (codepoint >> 12) as u8);

        let leading_octet = 0b1111_0000_u8;
        let leading_mask = !leading_octet;
        let leading_result = leading_octet | (leading_mask & (codepoint >> 18) as u8);

        return vec![
            leading_result,
            first_continuation_octet,
            second_continuation_octect,
            last_continuation_octet,
        ];
    }

    REPLACEMENT_CHAR_OCTETS.to_vec()
}

/// "lossy" because it doens't fail when an invalid sequence is passed in. It returns 0xFFFD
pub fn utf8_decode_lossy(s: Vec<u8>) -> Vec<u32> {
    let determine_n_octets_mask = 0b1111_0000_u32;
    let mut output = Vec::new();
    let mut curr_output = 0b0000_0000_u32;
    let mut total_n_octets = 0;
    let mut continuing_octet = 0;
    let mut i = 0;

    while i < s.len() {
        if continuing_octet > 0 {
            let shift = match continuing_octet {
                1 => 0,
                2 => 6,
                3 => 12,
                _ => panic!("shift overflow"),
            } as u32;

            let val = (s[i] as u32 & 0b0011_1111_u32) << shift;
            let octet = val | curr_output;
            curr_output = octet;

            if continuing_octet > 1 {
                continuing_octet -= 1;
                i += 1;
                continue;
            }

            if PROHIBITED_RANGE.contains(&curr_output) {
                output.push(REPLACEMENT_CHAR_0X);
                curr_output = 0b0000_0000_u32;
                continuing_octet -= 1;
                i += 1;
                continue;
            }

            let range = match total_n_octets {
                2 => TWO_OCTETS_RANGE.clone(),
                3 => THREE_OCTETS_RANGE.clone(),
                4 => FOUR_OCTETS_RANGE.clone(),
                _ => panic!("unsopported total octets"),
            };

            if range.contains(&curr_output) {
                output.push(curr_output);
                curr_output = 0b0000_0000_u32;
            } else {
                output.push(REPLACEMENT_CHAR_0X);
                curr_output = 0b0000_0000_u32;
            }

            continuing_octet -= 1;
            i += 1;
            continue;
        }

        let n_octets = s[i] as u32 & determine_n_octets_mask;

        match n_octets {
            0b1100_0000 => {
                let octet = (s[i] << 6) as u32 | curr_output;
                curr_output = octet;
                continuing_octet = 1;
                total_n_octets = 2;
                i += 1;
                continue;
            }
            0b1110_0000 => {
                let val = (s[i] & 0b0001_1111) as u32;
                let octet = (val << 12) | curr_output;
                curr_output = octet;
                continuing_octet = 2;
                total_n_octets = 3;
                i += 1;
                continue;
            }
            0b1111_0000 => {
                let val = (s[i] & 0b0000_1111) as u32;
                let octet = (val << 18) | curr_output;
                curr_output = octet;
                continuing_octet = 3;
                total_n_octets = 4;
                i += 1;
                continue;
            }
            val => {
                if val <= 0x7f {
                    let octet = s[i] as u32 & 0b0111_1111_u32;
                    output.push(octet);
                }
                i += 1;
                continue;
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_encode_ascii() {
        let res = utf8_encode_lossy(0x41);
        assert_eq!(res[0], 0b01000001);
    }

    #[test]
    fn utf8_encode_two_octets_range_test() {
        let res = utf8_encode_lossy(0xA1);
        assert_eq!(res[0], 0b11000010_u8);
        assert_eq!(res[1], 0b10100001_u8);
    }

    #[test]
    fn utf8_encode_three_octets_range_test() {
        let res = utf8_encode_lossy(0x0801);
        assert_eq!(res[0], 0b11100000_u8);
        assert_eq!(res[1], 0b10100000_u8);
        assert_eq!(res[2], 0b10000001_u8);
    }

    #[test]
    fn utf8_encode_four_octets_range_test() {
        let res = utf8_encode_lossy(0x10001);
        assert_eq!(res[0], 0b11110000_u8);
        assert_eq!(res[1], 0b10010000_u8);
        assert_eq!(res[2], 0b10000000_u8);
        assert_eq!(res[3], 0b10000001_u8);
    }

    #[test]
    fn utf8_encode_no_range() {
        let res = utf8_encode_lossy(0x11ffff);
        assert_eq!(res, REPLACEMENT_CHAR_OCTETS)
    }

    #[test]
    fn utf8_decode_ascii_test() {
        let res = utf8_encode_lossy(0x41);
        assert!(res.is_ascii());
        let out = utf8_decode_lossy(res);
        assert_eq!(out[0], 0b01000001);
    }

    #[test]
    fn utf8_decode_two_octets_test() {
        let res = utf8_encode_lossy(0xA1);
        let out = utf8_decode_lossy(res);
        assert_eq!(out[0], 0b10100001);
    }

    #[test]
    fn utf8_decode_three_octets_test() {
        let res = utf8_encode_lossy(0x0801);
        let out = utf8_decode_lossy(res);
        assert_eq!(out[0], 0b100000000001);
    }

    #[test]
    fn utf8_decode_four_octets_test() {
        let res = utf8_encode_lossy(0x10001);
        let out = utf8_decode_lossy(res);
        assert_eq!(out[0], 0b10000000000000001);
    }

    #[test]
    fn roundtrip_and_into_string_test() {
        let unicode_codepoints = Vec::<u32>::from(&[0x68, 0x65, 0x6c, 0x6c, 0x6f]);
        let encoded_utf8 = unicode_codepoints
            .into_iter()
            .flat_map(utf8_encode_lossy)
            .collect::<Vec<u8>>();

        let hello = String::from_utf8(encoded_utf8).unwrap();
        assert_eq!(hello, "hello");
    }

    #[test]
    fn invalid_encoding_test() {
        let encoded = utf8_encode_lossy(0xD802);
        assert_eq!(encoded, REPLACEMENT_CHAR_OCTETS);

        let encoded = utf8_encode_lossy(0x11ffff);
        assert_eq!(encoded, REPLACEMENT_CHAR_OCTETS);
    }

    #[test]
    fn invalid_sequences_decoding_test() {
        let decoded = utf8_decode_lossy(vec![0b11101101, 0b10100000, 0b10000000]); // 0xED 0xA0 0x80

        assert_eq!(decoded[0], REPLACEMENT_CHAR_0X);

        let decoded = utf8_decode_lossy(vec![
            0b11101101, 0b10100001, 0b10001100, 0b11101101, 0b10111110, 0b10110100,
        ]); // 0xED 0xA1 0x8C 0xED 0xBE 0xB4

        assert_eq!(decoded[0], REPLACEMENT_CHAR_0X);

        let decoded = utf8_decode_lossy(vec![0b11000000, 0b10000000]); // 0xC0 0x8

        assert_eq!(decoded[0], REPLACEMENT_CHAR_0X);
    }
}
