pub(crate) struct PineRegexCodePointEscape {
    pub(crate) end: usize,
    pub(crate) hex_width: usize,
    pub(crate) scalar: Option<char>,
}

fn parse_hex_scalar(digits: &str) -> Option<Option<char>> {
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }

    let significant = digits.trim_start_matches('0');
    let value = if significant.is_empty() {
        0
    } else {
        if significant.len() > 6 {
            return None;
        }
        u32::from_str_radix(significant, 16).ok()?
    };
    if value > 0x10_FFFF {
        return None;
    }
    Some(char::from_u32(value))
}

pub(crate) fn parse_pine_regex_code_point_escape(
    pattern: &str,
    index: usize,
) -> Option<PineRegexCodePointEscape> {
    let bytes = pattern.as_bytes();
    if bytes.get(index) != Some(&b'\\') {
        return None;
    }

    let kind = *bytes.get(index + 1)?;
    let digits_start = index + 2;
    let (digits, end, hex_width) = match kind {
        b'u' => (
            pattern.get(digits_start..digits_start + 4)?,
            digits_start + 4,
            4,
        ),
        b'x' if bytes.get(digits_start) == Some(&b'{') => {
            let digits_start = digits_start + 1;
            let digits_len = pattern.get(digits_start..)?.find('}')?;
            let end = digits_start + digits_len + 1;
            (
                pattern.get(digits_start..digits_start + digits_len)?,
                end,
                0,
            )
        }
        b'x' => (
            pattern.get(digits_start..digits_start + 2)?,
            digits_start + 2,
            2,
        ),
        _ => return None,
    };

    Some(PineRegexCodePointEscape {
        end,
        hex_width,
        scalar: parse_hex_scalar(digits)?,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_pine_regex_code_point_escape;

    #[test]
    fn parses_fixed_and_braced_code_point_escapes() {
        let fixed = parse_pine_regex_code_point_escape(r"\x6B0", 0).expect("fixed hex escape");
        assert_eq!(fixed.scalar, Some('k'));
        assert_eq!(fixed.end, 4);
        assert_eq!(fixed.hex_width, 2);

        let unicode =
            parse_pine_regex_code_point_escape(r"\u03B2x", 0).expect("fixed Unicode escape");
        assert_eq!(unicode.scalar, Some('β'));
        assert_eq!(unicode.end, 6);
        assert_eq!(unicode.hex_width, 4);

        let braced = parse_pine_regex_code_point_escape(r"\x{0000006B}0", 0)
            .expect("braced hex escape with leading zeros");
        assert_eq!(braced.scalar, Some('k'));
        assert_eq!(braced.end, 12);
        assert_eq!(braced.hex_width, 0);
    }

    #[test]
    fn distinguishes_surrogates_from_invalid_code_points() {
        let surrogate =
            parse_pine_regex_code_point_escape(r"\x{D800}", 0).expect("surrogate escape");
        assert_eq!(surrogate.scalar, None);

        assert!(parse_pine_regex_code_point_escape(r"\x{}", 0).is_none());
        assert!(parse_pine_regex_code_point_escape(r"\x{110000}", 0).is_none());
        assert!(parse_pine_regex_code_point_escape(r"\xG1", 0).is_none());
        assert!(parse_pine_regex_code_point_escape(r"\u123", 0).is_none());
    }
}
