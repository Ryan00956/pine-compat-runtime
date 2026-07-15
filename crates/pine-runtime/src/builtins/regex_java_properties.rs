pub(crate) fn pine_java_property(name: &str, negated: bool) -> Option<&'static str> {
    let (positive, negative) = match name {
        "javaLowerCase" => (r"\p{Lowercase}", r"\P{Lowercase}"),
        "javaUpperCase" => (r"\p{Uppercase}", r"\P{Uppercase}"),
        "javaAlphabetic" => (r"\p{Alphabetic}", r"\P{Alphabetic}"),
        "javaIdeographic" => (r"\p{Ideographic}", r"\P{Ideographic}"),
        "javaTitleCase" => (r"\p{Lt}", r"\P{Lt}"),
        "javaDigit" => (r"\p{Nd}", r"\P{Nd}"),
        "javaDefined" => (r"\P{Cn}", r"\p{Cn}"),
        "javaLetter" => (r"\p{L}", r"\P{L}"),
        "javaLetterOrDigit" => (r"[\p{L}\p{Nd}]", r"[^\p{L}\p{Nd}]"),
        _ => return None,
    };
    Some(if negated { negative } else { positive })
}

#[cfg(test)]
mod tests {
    use super::pine_java_property;

    #[test]
    fn maps_basic_java_character_properties_and_complements() {
        assert_eq!(
            pine_java_property("javaLowerCase", false),
            Some(r"\p{Lowercase}")
        );
        assert_eq!(pine_java_property("javaDefined", true), Some(r"\p{Cn}"));
        assert_eq!(
            pine_java_property("javaLetterOrDigit", false),
            Some(r"[\p{L}\p{Nd}]")
        );
        assert_eq!(pine_java_property("javalowercase", false), None);
    }
}
