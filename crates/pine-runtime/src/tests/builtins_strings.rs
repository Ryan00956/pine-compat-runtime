use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use regex::Regex;

use crate::builtins::strings::normalize_pine_regex;

use super::*;

#[test]
fn normalizes_pine_regex_unicode_class_modes() {
    assert_eq!(
        normalize_pine_regex(r"\d(?U)\w(?-U)\s"),
        r"[0-9]\w[ \t\n\x0B\f\r]"
    );
    assert_eq!(
        normalize_pine_regex(r"(?iU:\d)(?-U:\w)"),
        r"(?i:\d)(?:[A-Za-z0-9_])"
    );
    assert_eq!(
        normalize_pine_regex("(?x)\\d # (?U)\\w\n(?U)\\w"),
        "(?x)[0-9] # (?U)\\w\n\\w"
    );
    assert_eq!(normalize_pine_regex("(?)"), "(?)");
    assert_eq!(normalize_pine_regex("(?-)"), "(?-)");
}

#[test]
fn normalizes_pine_regex_horizontal_whitespace_classes() {
    let horizontal =
        Regex::new(&normalize_pine_regex(r"^\h$")).expect("normalized horizontal whitespace regex");
    for ch in [
        ' ', '\t', '\u{00a0}', '\u{1680}', '\u{180e}', '\u{2000}', '\u{200a}', '\u{202f}',
        '\u{205f}', '\u{3000}',
    ] {
        assert!(horizontal.is_match(&ch.to_string()), "U+{:04X}", ch as u32);
    }
    assert!(!horizontal.is_match("\n"));

    let non_horizontal = Regex::new(&normalize_pine_regex(r"^\H$"))
        .expect("normalized non-horizontal whitespace regex");
    assert!(non_horizontal.is_match("A"));
    assert!(non_horizontal.is_match("\n"));
    assert!(!non_horizontal.is_match("\u{2003}"));

    assert_eq!(
        normalize_pine_regex(r"(?U:\h)[\H](?-U:\h)"),
        r"(?:[ \t\x{00A0}\x{1680}\x{180E}\x{2000}-\x{200A}\x{202F}\x{205F}\x{3000}])[[^ \t\x{00A0}\x{1680}\x{180E}\x{2000}-\x{200A}\x{202F}\x{205F}\x{3000}]](?:[ \t\x{00A0}\x{1680}\x{180E}\x{2000}-\x{200A}\x{202F}\x{205F}\x{3000}])"
    );
}

#[test]
fn normalizes_pine_regex_quoted_literals() {
    assert_eq!(
        normalize_pine_regex(r"\Q(?U)\d[.]# \E\d"),
        r"\x{28}\x{3F}\x{55}\x{29}\x{5C}\x{64}\x{5B}\x{2E}\x{5D}\x{23}\x{20}[0-9]"
    );

    let verbose = Regex::new(&normalize_pine_regex(r"(?x)^\Q# [ ]\E$"))
        .expect("normalized verbose quoted regex");
    assert!(verbose.is_match("# [ ]"));

    let in_class = Regex::new(&normalize_pine_regex(r"^[\Q]-\E]+$"))
        .expect("normalized quoted character class regex");
    assert!(in_class.is_match("]-"));

    let unclosed =
        Regex::new(&normalize_pine_regex(r"^\Q[a]+$")).expect("normalized unclosed quoted regex");
    assert!(unclosed.is_match("[a]+$"));
}

#[test]
fn normalizes_pine_regex_four_digit_unicode_escapes() {
    assert_eq!(
        normalize_pine_regex(r"\u2014[\u00E5]\u00610"),
        r"\x{2014}[\x{00E5}]\x{0061}0"
    );
    assert_eq!(normalize_pine_regex(r"\u123"), r"\u123");
    assert_eq!(
        normalize_pine_regex(r"\Q\u2014\E\u2014"),
        r"\x{5C}\x{75}\x{32}\x{30}\x{31}\x{34}\x{2014}"
    );
}

#[test]
fn normalizes_pine_regex_dot_line_terminators() {
    let default_dot =
        Regex::new(&normalize_pine_regex(r"\A.\z")).expect("normalized default-dot regex");
    for ch in ['\n', '\r', '\u{0085}', '\u{2028}', '\u{2029}'] {
        assert!(
            !default_dot.is_match(&ch.to_string()),
            "U+{:04X}",
            ch as u32
        );
    }
    for ch in ['A', '\u{000b}', '\u{000c}'] {
        assert!(default_dot.is_match(&ch.to_string()), "U+{:04X}", ch as u32);
    }

    let dotall = Regex::new(&normalize_pine_regex(r"(?s:\A.\z)")).expect("normalized dotall regex");
    for ch in ['\n', '\r', '\u{0085}', '\u{2028}', '\u{2029}'] {
        assert!(dotall.is_match(&ch.to_string()), "U+{:04X}", ch as u32);
    }

    assert_eq!(
        normalize_pine_regex(r".(?s).(?-s).(?s:.)(?-s:.)"),
        r"[^\n\r\x{0085}\x{2028}\x{2029}](?s).(?-s)[^\n\r\x{0085}\x{2028}\x{2029}](?s:.)(?-s:[^\n\r\x{0085}\x{2028}\x{2029}])"
    );
    assert_eq!(normalize_pine_regex(r"\.[.]\Q.\E"), r"\.[.]\x{2E}");
}

#[test]
fn normalizes_pine_regex_posix_classes() {
    assert_eq!(
        normalize_pine_regex(r"\p{Lower}\P{XDigit}(?U)\p{Lower}(?-U)\p{XDigit}"),
        r"[a-z][^0-9A-Fa-f]\p{Lowercase}[0-9A-Fa-f]"
    );
    assert_eq!(
        normalize_pine_regex(r"\p{L}\p{Lowercase}\p{Unknown}"),
        r"\p{L}\p{Lowercase}\p{Unknown}"
    );
    assert_eq!(
        normalize_pine_regex(r"\p{alnum}(?U)\p{aLnUm}"),
        r"\p{alnum}[\p{Alphabetic}\p{Nd}]"
    );
    assert!(Regex::new(&normalize_pine_regex(r"\p{alnum}")).is_err());
    assert_eq!(
        normalize_pine_regex(r"\Q\p{Lower}\E\p{Lower}"),
        r"\x{5C}\x{70}\x{7B}\x{4C}\x{6F}\x{77}\x{65}\x{72}\x{7D}[a-z]"
    );

    for name in [
        "Lower", "Upper", "ASCII", "Alpha", "Digit", "Alnum", "Punct", "Graph", "Print", "Blank",
        "Cntrl", "XDigit", "Space",
    ] {
        for prefix in [r"\p", r"\P"] {
            let default_pattern = format!(r"{prefix}{{{name}}}");
            Regex::new(&normalize_pine_regex(&default_pattern))
                .unwrap_or_else(|err| panic!("default {prefix}{{{name}}}: {err}"));
            let unicode_pattern = format!(r"(?U:{prefix}{{{name}}})");
            Regex::new(&normalize_pine_regex(&unicode_pattern))
                .unwrap_or_else(|err| panic!("Unicode {prefix}{{{name}}}: {err}"));
        }
    }

    let default_lower = Regex::new(&normalize_pine_regex(r"\p{Lower}"))
        .expect("normalized default POSIX lower regex");
    assert_eq!(
        default_lower.find("βa").map(|matched| matched.as_str()),
        Some("a")
    );
    let unicode_lower = Regex::new(&normalize_pine_regex(r"(?U)\p{Lower}"))
        .expect("normalized Unicode POSIX lower regex");
    assert_eq!(
        unicode_lower.find("βa").map(|matched| matched.as_str()),
        Some("β")
    );

    let default_xdigit = Regex::new(&normalize_pine_regex(r"[\p{XDigit}]"))
        .expect("normalized default POSIX xdigit regex");
    assert_eq!(
        default_xdigit.find("𝟙F").map(|matched| matched.as_str()),
        Some("F")
    );
    let unicode_xdigit = Regex::new(&normalize_pine_regex(r"(?U)[\p{XDigit}]"))
        .expect("normalized Unicode POSIX xdigit regex");
    assert_eq!(
        unicode_xdigit.find("𝟙F").map(|matched| matched.as_str()),
        Some("𝟙")
    );

    let unicode_graph = Regex::new(&normalize_pine_regex(r"(?U)\p{Graph}"))
        .expect("normalized Unicode POSIX graph regex");
    assert!(unicode_graph.is_match("—"));
    assert!(unicode_graph.is_match("\u{200d}"));
    assert!(!unicode_graph.is_match("\u{00a0}"));
    assert!(!unicode_graph.is_match("\u{0378}"));
    let unicode_print = Regex::new(&normalize_pine_regex(r"(?U)\p{Print}"))
        .expect("normalized Unicode POSIX print regex");
    assert!(unicode_print.is_match("\u{00a0}"));
    assert!(!unicode_print.is_match("\t"));
}

#[test]
fn runs_string_helpers() {
    let source = SourceFile::new(
        "test.pine",
        r##"indicator("strings")
mode = input.string("sma", "Mode")
upper = str.upper(mode)
lower = str.lower(upper)
length = str.length(upper)
missing = str.length(na)
missing_upper = str.upper(na)
missing_lower = str.lower(na)
unicode_length = str.length("åβ")
unicode_upper = str.upper("åβ")
unicode_lower = str.lower("ÅΒ")
empty_length = str.length("")
matched = str.contains(upper, "M") and str.startswith(upper, "S") and str.endswith(upper, "A")
not_matched = not str.contains(upper, "Z") and not str.startswith(upper, "M") and not str.endswith(upper, "S")
empty_match = str.contains(upper, "") and str.startswith(upper, "") and str.endswith(upper, "")
empty_source_match = str.contains("", "") and str.startswith("", "") and str.endswith("", "")
missing_match = str.contains(na, "S")
missing_pattern_match = str.contains(upper, na)
missing_pattern_start = str.startswith(upper, na)
missing_pattern_end = str.endswith(upper, na)
mid = str.pos(upper, "M")
missing_pos = str.pos(upper, "Z")
empty_pos = str.pos(upper, "")
na_pos = str.pos(upper, na)
na_source_pos = str.pos(na, "S")
unicode_pos = str.pos("åβγ", "γ")
slice = str.substring(upper, mid, mid + 1)
tail = str.substring(upper, mid)
wide = str.substring(upper, 1, 99)
empty_slice = str.substring(upper, 1, 1)
tail_empty_slice = str.substring(upper, str.length(upper))
na_source_slice = str.substring(na, 0, 1)
na_begin = str.substring(upper, na, 1)
na_end = str.substring(upper, 1, na)
unicode_slice = str.substring("åβ", 1, 2)
trimmed = str.trim(" \tSMA\n")
non_ascii_trimmed = str.trim(" SMA ")
missing_trim = str.trim(na)
empty_trim = str.trim(" \t\n")
repeated = str.repeat("ab", 2, "-")
default_repeat = str.repeat("ab", 2)
empty_repeat = str.repeat("ab", 0)
missing_repeat = str.repeat("ab", na)
missing_repeat_source = str.repeat(na, 2)
missing_repeat_separator = str.repeat("ab", 2, na)
replace_first = str.replace("hello", "l", "1")
replace_second = str.replace("hello", "l", "1", 1)
replace_missing = str.replace("hello", "z", "1", 0)
replace_negative = str.replace("hello", "l", "1", -1)
replace_na_occurrence = str.replace("hello", "l", "1", na)
replace_out_of_range = str.replace("hello", "l", "1", 9)
replace_all = str.replace_all("hello", "l", "1")
replace_all_missing = str.replace_all("hello", "z", "1")
replace_boundary = str.replace("ab", "", ".", 1)
replace_all_boundaries = str.replace_all("ab", "", ".")
missing_replace = str.replace(na, "x", "y")
missing_replace_target = str.replace("hello", na, "y")
missing_replace_replacement = str.replace("hello", "x", na)
missing_replace_all_target = str.replace_all("hello", na, "y")
missing_replace_all_replacement = str.replace_all("hello", "x", na)
number = str.tonumber("1234.50")
signed_number = str.tonumber("-.5")
plus_number = str.tonumber("+12")
dot_number = str.tonumber(".5")
invalid_number = str.tonumber("$1,234.50")
empty_number = str.tonumber("")
whitespace_number = str.tonumber(" 1")
exponent_number = str.tonumber("1e3")
signed_exponent_number = str.tonumber("-.5e+2")
upper_exponent_number = str.tonumber("+12E-1")
double_decimal_number = str.tonumber("1.2.3")
bad_exponent_number = str.tonumber("1e")
nonfinite_exponent_number = str.tonumber("1e309")
missing_number = str.tonumber(na)
text_int = str.tostring(42)
text_float = str.tostring(1.25)
text_na_format = str.tostring(1.25, na)
text_round0 = str.tostring(1.25, "#")
text_round1 = str.tostring(1.25, "#.#")
text_zeros = str.tostring(1.25, "#.0000")
text_percent = str.tostring(12.3456, format.percent)
text_percent_unscaled = str.tostring(0.1234, format.percent)
text_percent_custom_scaled = str.tostring(0.1234, "#.##%")
text_price = str.tostring(1.234567891, format.price)
text_volume_small = str.tostring(518.3, format.volume)
text_volume_k = str.tostring(5183, format.volume)
text_volume_m = str.tostring(2500000, format.volume)
text_volume_b = str.tostring(2500000000, format.volume)
text_volume_t = str.tostring(1250000000000, format.volume)
text_volume_negative = str.tostring(-5183, format.volume)
text_volume_boundary = str.tostring(999.5, format.volume)
text_mintick_down = str.tostring(1.234, format.mintick)
text_mintick_up = str.tostring(1.235, format.mintick)
text_mintick_negative_tie = str.tostring(-1.235, format.mintick)
text_mintick_trailing_zeros = str.tostring(1, format.mintick)
text_bool = str.tostring(true)
text_bool_false = str.tostring(false)
text_string = str.tostring("ok")
text_na = str.tostring(na)
values = array.new_float(3)
array.set(values, 0, 1.2)
array.set(values, 1, 2.6)
text_array = str.tostring(values, "#")
formatted = str.format("A={0}, B={1}, A2={0}", text_int, text_float)
formatted_missing = str.format("Missing {2}", text_int)
formatted_number = str.format("Rounded {0,number,#.00}", 1.2)
formatted_percent_preset = str.format("Percent {0,number,percent}", 0.0345)
formatted_percent_grouped = str.format("Percent {0,number,percent}", 1234.56)
formatted_percent_custom = str.format("Percent {0,number,#.##%}", 0.0345)
formatted_integer = str.format("Integer {0,number,integer}", 1234.5)
formatted_currency = str.format("Currency {0,number,currency}", 1234.5)
formatted_array = str.format("Values {0}", values)
formatted_datetime = str.format("{0,date,yyyy-MM-dd}T{0,time,HH:mm:ssZ}", 1609459200000)
formatted_datetime_tokens = str.format("{0,date,D E w W} {0,time,HH:mm:ssZ}", 1609459200000)
formatted_datetime_clock_tokens = str.format("{0,time,H HH h hh:mm:ss.S SSS a}", 1609506245123)
formatted_quote = str.format("Literal '{0}' and apostrophe '' {0}", "X")
formatted_na = str.format(na, text_int)
formatted_na_arg = str.format("Missing {0}", na)
formatted_bool = str.format("Flag {0}", true)
match_prefix = str.match("NASDAQ:AAPL", "^(?:BATS|NASDAQ|NYSE|AMEX):")
match_suffix = str.match("NASDAQ:AAPL", "AAPL$")
match_missing = str.match("NASDAQ:AAPL", "^NYSE:")
match_ascii_digit = str.match("１1", "\\d")
match_unicode_digit = str.match("１1", "(?U)\\d")
match_ascii_non_digit = str.match("１1", "\\D")
match_unicode_non_digit = str.match("１A", "(?U)\\D")
match_ascii_word = str.match("β_A", "\\w+")
match_unicode_word = str.match("β_A", "(?U)\\w+")
match_ascii_non_word = str.match("βA", "\\W")
match_unicode_non_word = str.match("β!", "(?U)\\W")
match_ascii_space = str.match("  ", "\\s")
match_unicode_space = str.match("  ", "(?U)\\s")
match_ascii_non_space = str.match("  ", "\\S")
match_ascii_boundary = str.match("βA", "\\bA")
match_unicode_boundary = str.match("βA", "(?U)\\bA")
match_ascii_non_boundary = str.match("βA", "\\BA")
match_unicode_non_boundary = str.match("βA", "(?U)\\BA")
match_ascii_class = str.match("β1", "[\\w]+")
match_scoped_unicode_digit = str.match("１", "(?U:\\d)")
match_scoped_unicode = str.match("１１", "(?U:\\d)\\d")
match_toggled_unicode = str.match("１2", "(?U)\\d(?-U)\\d")
match_unicode_greedy = str.match("abc", "(?U).+")
match_horizontal_tab = str.match("\tA", "\\h")
match_horizontal_em_space = str.match(" A", "\\h")
match_non_horizontal = str.match(" A", "\\H")
match_horizontal_unicode_on = str.match(" ", "(?U)\\h")
match_horizontal_unicode_off = str.match(" ", "(?-U)\\h")
match_horizontal_class = str.match("A ", "[\\h]")
match_quoted_meta = str.match("x[a-z]+(?U)# y", "\\Q[a-z]+(?U)# \\E")
match_quoted_escape = str.match("\\d123", "\\Q\\d\\E")
match_quoted_verbose = str.match("# [ ]", "(?x)\\Q# [ ]\\E")
match_quoted_class = str.match("]-", "[\\Q]-\\E]+")
match_quoted_unclosed = str.match("[b]+", "\\Q[b]+")
match_quoted_then_ascii = str.match("[a]123", "\\Q[a]\\E\\d+")
match_final_newline_dollar = str.match("tail\n", "tail$")
match_final_newline_Z = str.match("tail\n", "tail\\Z")
match_absolute_z_missing = str.match("tail\n", "tail\\z")
match_absolute_z = str.match("tail", "tail\\z")
match_explicit_final_newline = str.match("tail\n", "\\n$")
match_empty_final_newline_dollar = str.match("\n", "$")
match_empty_final_newline_Z = str.match("\n", "\\Z")
match_multiline_dollar = str.match("first\nsecond", "(?m)^second$")
match_scoped_multiline_reset = str.match("first\nsecond\n", "(?m:first$)\\n(?-m:second$)")
match_dotall_greedy_end = str.match("a\n", "(?s).*$")
match_dotall_lazy_end = str.match("a\n", "(?s).*?$")
match_default_dot_line_feed = str.match("\nA", ".")
match_default_dot_carriage_return = str.match("\rA", ".")
match_default_dot_next_line = str.match("A", ".")
match_default_dot_line_separator = str.match(" A", ".")
match_default_dot_paragraph_separator = str.match(" A", ".")
match_dotall_line_terminators = str.match("\r  A", "(?s).+")
match_global_dotall_reset = str.match("\rA", "(?s).(?-s).")
match_scoped_dotall_reset = str.match("\rA", "(?s:.)(?-s:.)")
match_literal_dots = str.match("...", "\\.[.]\\Q.\\E")
match_posix_lower_ascii = str.match("βa", "\\p{Lower}")
match_posix_lower_unicode = str.match("βa", "(?U)\\p{Lower}")
match_posix_not_lower_ascii = str.match("βa", "\\P{Lower}")
match_posix_not_lower_unicode = str.match("βA", "(?U)\\P{Lower}")
match_posix_xdigit_ascii = str.match("𝟙F", "[\\p{XDigit}]")
match_posix_xdigit_unicode = str.match("𝟙F", "(?U)[\\p{XDigit}]")
match_posix_scoped_reset = str.match("βa", "(?U:\\p{Lower})(?-U:\\p{Lower})")
match_posix_unicode_casefold_name = str.match("β", "(?U)\\p{aLpHa}")
match_unicode_category_unchanged = str.match("β", "\\p{L}")
match_posix_quoted = str.match("\\p{Lower}", "\\Q\\p{Lower}\\E")
match_anchor_capture_collision = str.match("tail\n", "(?<__pine_final_newline_0>tail)$")
match_unicode_escape = str.match("x—y", "\\u2014")
match_unicode_escape_class = str.match("xåy", "[\\u00E5]")
match_unicode_escape_fifth_digit = str.match("xa0y", "\\u00610")
match_unicode_escape_quoted = str.match("\\u2014—", "\\Q\\u2014\\E\\u2014")
missing_match_regex = str.match(na, ".+")
missing_match_pattern = str.match("NASDAQ:AAPL", na)
split_words = str.split("A,B,,C", ",")
split_missing_separator_literal = str.split("A,B", "|")
split_chars = str.split("xy", "")
split_unicode = str.split("åβ", "")
split_empty_source_separator = str.split("", ",")
split_empty_source_chars = str.split("", "")
split_missing = str.split(na, ",")
split_missing_separator = str.split("A,B", na)
formatted_time_default = str.format_time(1609459200000)
formatted_time_date = str.format_time(1609459200000, "yyyy-MM-dd")
formatted_time_text = str.format_time(1609459200000, "HH:mm:ss 'on' MMM dd, yyyy", "UTC")
formatted_time_alias = str.format_time(1609459200000, "HH:mm:ssZ", "UTC+00:00")
formatted_time_gmt_alias = str.format_time(1609459200000, "HH:mm:ssZ", "GMT-0")
formatted_time_fixed_east = str.format_time(1609459200000, "yyyy-MM-dd HH:mm:ssZ", "UTC+4")
formatted_time_fixed_west = str.format_time(1609459200000, "yyyy-MM-dd HH:mm:ssZ", "GMT-5")
formatted_time_numeric_offset = str.format_time(1609459200000, "HH:mm:ssZ", "+05:30")
formatted_time_day_of_year = str.format_time(1609459200000, "D DD DDD", "UTC")
formatted_time_day_of_year_later = str.format_time(1612235045000, "D DDD", "UTC")
formatted_time_weekday = str.format_time(1609459200000, "E EEEE", "UTC")
formatted_time_weekday_later = str.format_time(1612235045000, "EEE EEEE", "UTC")
formatted_time_week_of_year = str.format_time(1609459200000, "w ww", "UTC")
formatted_time_week_of_year_later = str.format_time(1612235045000, "w ww", "UTC")
formatted_time_week_of_month = str.format_time(1609459200000, "W WW", "UTC")
formatted_time_week_of_month_later = str.format_time(1612742400000, "W WW", "UTC")
formatted_time_clock_tokens = str.format_time(1609506245123, "H HH h hh:mm:ss.S SSS a", "UTC")
formatted_time_na_format = str.format_time(1609459200000, na)
formatted_time_na_timezone = str.format_time(1609459200000, "HH:mm:ssZ", na)
missing_format_time = str.format_time(na)
plot(upper == "SMA" and lower == "sma" and unicode_length == 2 and unicode_upper == "åβ" and unicode_lower == "ÅΒ" and empty_length == 0 ? length : 0)
plot(na(missing) and na(missing_upper) and na(missing_lower) ? 1 : 0)
plot(matched and not_matched and empty_match and empty_source_match ? 1 : 0)
plot(na(missing_match) and na(missing_pattern_match) and na(missing_pattern_start) and na(missing_pattern_end) ? 1 : 0)
plot(unicode_pos == 2 ? mid + empty_pos : 0)
plot(na(na_pos) ? 0 : na_pos == 0 and na(na_source_pos) ? 1 : 0)
plot(na(missing_pos) ? 1 : 0)
plot(slice == "M" and tail == "MA" and wide == "MA" and empty_slice == "" and tail_empty_slice == "" and na(na_source_slice) and na_begin == "S" and na_end == "MA" and unicode_slice == "β" ? 1 : 0)
plot(trimmed == upper and non_ascii_trimmed == " SMA " and missing_trim == "" and empty_trim == "" and repeated == "ab-ab" and default_repeat == "abab" and empty_repeat == "" ? 1 : 0)
plot(na(missing_repeat) and na(missing_repeat_source) and na(missing_repeat_separator) ? 1 : 0)
plot(replace_first == "he1lo" and replace_second == "hel1o" and replace_missing == "hello" and replace_negative == "hello" and replace_na_occurrence == "he1lo" and replace_out_of_range == "hello" ? 1 : 0)
plot(replace_all == "he11o" and replace_all_missing == "hello" and replace_boundary == "a.b" and replace_all_boundaries == ".a.b." ? 1 : 0)
plot(na(missing_replace) and na(missing_replace_target) and na(missing_replace_replacement) and na(missing_replace_all_target) and na(missing_replace_all_replacement) ? 1 : 0)
plot(number == 1234.5 and signed_number == -0.5 and plus_number == 12 and dot_number == 0.5 ? 1 : 0)
plot(exponent_number == 1000 and signed_exponent_number == -50 and upper_exponent_number == 1.2 ? 1 : 0)
plot(na(invalid_number) and na(empty_number) and na(whitespace_number) and na(double_decimal_number) and na(bad_exponent_number) and na(nonfinite_exponent_number) and na(missing_number) ? 1 : 0)
plot(text_int == "42" and text_float == "1.25" and text_na_format == "1.25" and text_round0 == "1" and text_round1 == "1.3" ? 1 : 0)
plot(text_zeros == "1.2500" and text_percent == "12.35%" and text_percent_unscaled == "0.12%" and text_percent_custom_scaled == "12.34%" ? 1 : 0)
plot(text_price == "1.23456789" and text_volume_small == "518" and text_volume_k == "5.183K" and text_volume_m == "2.5M" and text_volume_b == "2.5B" and text_volume_t == "1.25T" and text_volume_negative == "-5.183K" and text_volume_boundary == "1K" ? 1 : 0)
plot(text_bool == "true" and text_bool_false == "false" and text_string == "ok" and text_na == "NaN" ? 1 : 0)
plot(text_array == "[1, 3, NaN]" ? 1 : 0)
plot(formatted == "A=42, B=1.25, A2=42" and formatted_missing == "Missing {2}" ? 1 : 0)
plot(formatted_number == "Rounded 1.20" and formatted_percent_preset == "Percent 3%" and formatted_percent_grouped == "Percent 123,456%" and formatted_percent_custom == "Percent 3.45%" and formatted_integer == "Integer 1,235" and formatted_currency == "Currency $1,234.50" ? 1 : 0)
plot(formatted_array == "Values [1.2, 2.6, NaN]" and formatted_datetime == "2021-01-01T00:00:00+0000" and formatted_datetime_tokens == "1 Fri 53 1 00:00:00+0000" and formatted_datetime_clock_tokens == "13 13 1 01:04:05.123 123 PM" and formatted_quote == "Literal {0} and apostrophe ' X" and na(formatted_na) and formatted_na_arg == "Missing NaN" and formatted_bool == "Flag true" ? 1 : 0)
plot(match_prefix == "NASDAQ:" and match_suffix == "AAPL" and match_missing == "" ? 1 : 0)
plot(match_ascii_digit == "1" and match_unicode_digit == "１" and match_ascii_non_digit == "１" and match_unicode_non_digit == "A" ? 1 : 0)
plot(match_ascii_word == "_A" and match_unicode_word == "β_A" and match_ascii_non_word == "β" and match_unicode_non_word == "!" ? 1 : 0)
plot(match_ascii_space == " " and match_unicode_space == " " and match_ascii_non_space == " " ? 1 : 0)
plot(match_ascii_boundary == "A" and match_unicode_boundary == "" and match_ascii_non_boundary == "" and match_unicode_non_boundary == "A" and match_ascii_class == "1" ? 1 : 0)
plot(match_scoped_unicode_digit == "１" and match_scoped_unicode == "" and match_toggled_unicode == "１2" and match_unicode_greedy == "abc" ? 1 : 0)
plot(match_horizontal_tab == "\t" and match_horizontal_em_space == " " and match_non_horizontal == "A" and match_horizontal_unicode_on == " " and match_horizontal_unicode_off == " " and match_horizontal_class == " " ? 1 : 0)
plot(match_quoted_meta == "[a-z]+(?U)# " and match_quoted_escape == "\\d" and match_quoted_verbose == "# [ ]" and match_quoted_class == "]-" and match_quoted_unclosed == "[b]+" and match_quoted_then_ascii == "[a]123" ? 1 : 0)
plot(match_final_newline_dollar == "tail" and match_final_newline_Z == "tail" and match_absolute_z_missing == "" and match_absolute_z == "tail" ? 1 : 0)
plot(match_explicit_final_newline == "\n" and match_empty_final_newline_dollar == "" and match_empty_final_newline_Z == "" ? 1 : 0)
plot(match_multiline_dollar == "second" and match_scoped_multiline_reset == "first\nsecond" ? 1 : 0)
plot(match_dotall_greedy_end == "a\n" and match_dotall_lazy_end == "a" and match_anchor_capture_collision == "tail" ? 1 : 0)
plot(match_default_dot_line_feed == "A" and match_default_dot_carriage_return == "A" and match_default_dot_next_line == "A" and match_default_dot_line_separator == "A" and match_default_dot_paragraph_separator == "A" ? 1 : 0)
plot(match_dotall_line_terminators == "\r  A" and match_global_dotall_reset == "\rA" and match_scoped_dotall_reset == "\rA" and match_literal_dots == "..." ? 1 : 0)
plot(match_posix_lower_ascii == "a" and match_posix_lower_unicode == "β" and match_posix_not_lower_ascii == "β" and match_posix_not_lower_unicode == "A" ? 1 : 0)
plot(match_posix_xdigit_ascii == "F" and match_posix_xdigit_unicode == "𝟙" and match_posix_scoped_reset == "βa" and match_posix_unicode_casefold_name == "β" and match_unicode_category_unchanged == "β" and match_posix_quoted == "\\p{Lower}" ? 1 : 0)
plot(match_unicode_escape == "—" and match_unicode_escape_class == "å" and match_unicode_escape_fifth_digit == "a0" and match_unicode_escape_quoted == "\\u2014—" ? 1 : 0)
plot(na(missing_match_regex) and na(missing_match_pattern) ? 1 : 0)
plot(split_words.size() == 4 and split_words.get(0) == "A" and split_words.get(2) == "" and split_words.get(3) == "C" and split_missing_separator_literal.size() == 1 and split_missing_separator_literal.get(0) == "A,B" ? 1 : 0)
plot(split_chars.size() == 2 and split_chars.get(0) == "x" and split_chars.get(1) == "y" and split_unicode.size() == 2 and split_unicode.get(0) == "å" and split_unicode.get(1) == "β" and split_empty_source_separator.size() == 1 and split_empty_source_separator.get(0) == "" and split_empty_source_chars.size() == 0 and na(split_missing) and na(split_missing_separator) ? 1 : 0)
plot(formatted_time_default == "2021-01-01T00:00:00+0000" and formatted_time_date == "2021-01-01" and formatted_time_na_format == "2021-01-01T00:00:00+0000" ? 1 : 0)
plot(formatted_time_text == "00:00:00 on Jan 01, 2021" and missing_format_time == "1970-01-01T00:00:00+0000" ? 1 : 0)
plot(formatted_time_alias == "00:00:00+0000" and formatted_time_gmt_alias == "00:00:00+0000" and formatted_time_na_timezone == "00:00:00+0000" ? 1 : 0)
plot(formatted_time_fixed_east == "2021-01-01 04:00:00+0400" and formatted_time_fixed_west == "2020-12-31 19:00:00-0500" and formatted_time_numeric_offset == "05:30:00+0530" ? 1 : 0)
plot(formatted_time_day_of_year == "1 01 001" and formatted_time_day_of_year_later == "33 033" ? 1 : 0)
plot(formatted_time_weekday == "Fri Friday" and formatted_time_weekday_later == "Tue Tuesday" ? 1 : 0)
plot(formatted_time_week_of_year == "53 53" and formatted_time_week_of_year_later == "5 05" ? 1 : 0)
plot(formatted_time_week_of_month == "1 01" and formatted_time_week_of_month_later == "2 02" and formatted_time_clock_tokens == "13 13 1 01:04:05.123 123 PM" ? 1 : 0)
plot(text_mintick_down == "1.23" and text_mintick_up == "1.24" and text_mintick_negative_tie == "-1.23" and text_mintick_trailing_zeros == "1.00" ? 1 : 0)
string_values = array.from("head", "tail")
plot(str.tostring(string_values) == "[head, tail]" ? 1 : 0)
int_values = array.from(1, 2)
plot(str.tostring(int_values) == "[1, 2]" ? 1 : 0)
bool_values = array.from(true, false)
plot(str.tostring(bool_values) == "[true, false]" ? 1 : 0)
float_matrix = matrix.new<float>(2, 2)
matrix.set(float_matrix, 0, 0, 1.25)
matrix.set(float_matrix, 0, 1, 2.5)
matrix.set(float_matrix, 1, 1, -3.0)
int_matrix = matrix.new<int>(1, 2, 0)
matrix.set(int_matrix, 0, 1, -2)
bool_matrix = matrix.new<bool>(1, 2, false)
matrix.set(bool_matrix, 0, 1, true)
string_matrix = matrix.new<string>(2, 1, "")
matrix.set(string_matrix, 0, 0, "head")
matrix.set(string_matrix, 1, 0, "tail")
empty_rows_matrix = matrix.new<float>(0, 2)
empty_columns_matrix = matrix.new<float>(2, 0)
plot(str.tostring(float_matrix, "#.0") == "[[1.3, 2.5], [NaN, -3.0]]" and str.tostring(int_matrix) == "[[0, -2]]" and str.tostring(bool_matrix) == "[[false, true]]" and str.tostring(string_matrix) == "[[head], [tail]]" and str.tostring(empty_rows_matrix) == "[]" and str.tostring(empty_columns_matrix) == "[[], []]" ? 1 : 0)
mintick_values = array.from(1.234, 1.235, 1.0)
mintick_matrix = matrix.new<float>(1, 3)
matrix.set(mintick_matrix, 0, 0, 1.234)
matrix.set(mintick_matrix, 0, 1, 1.235)
matrix.set(mintick_matrix, 0, 2, 1.0)
plot(str.tostring(mintick_values, format.mintick) == "[1.23, 1.24, 1.00]" and str.tostring(mintick_matrix, format.mintick) == "[[1.23, 1.24, 1.00]]" ? 1 : 0)
volume_values = array.from(518.3, 5183.0, 2500000.0)
volume_matrix = matrix.new<float>(1, 3)
matrix.set(volume_matrix, 0, 0, 518.3)
matrix.set(volume_matrix, 0, 1, 5183.0)
matrix.set(volume_matrix, 0, 2, 2500000.0)
plot(str.tostring(volume_values, format.volume) == "[518, 5.183K, 2.5M]" and str.tostring(volume_matrix, format.volume) == "[[518, 5.183K, 2.5M]]" ? 1 : 0)
percent_values = array.from(12.3456, 0.1234)
percent_matrix = matrix.new<float>(1, 2)
matrix.set(percent_matrix, 0, 0, 12.3456)
matrix.set(percent_matrix, 0, 1, 0.1234)
plot(str.tostring(percent_values, format.percent) == "[12.35%, 0.12%]" and str.tostring(percent_matrix, format.percent) == "[[12.35%, 0.12%]]" ? 1 : 0)
plot(str.format("Words {0}", string_values) == "Words [head, tail]" ? 1 : 0)
plot(str.format("Ints {0}", int_values) == "Ints [1, 2]" ? 1 : 0)
plot(str.format("Flags {0}", bool_values) == "Flags [true, false]" ? 1 : 0)
"##,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[3.0, 3.0]);
    for plot in &result.plots[1..] {
        assert_values_close(&plot.values, &[1.0, 1.0]);
    }
}

#[test]
fn rejects_unbalanced_str_format_placeholders() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad format")
plot(str.length(str.format("Value {0", close)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected str.format placeholder error");

    assert!(
        error.message.contains("str.format has unmatched `{`"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_invalid_str_match_regex() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad match")
plot(str.length(str.match("abc", "(")))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected str.match regex error");

    assert!(
        error.message.contains("str.match invalid regex"),
        "{}",
        error.message
    );
}

#[test]
fn formats_iana_str_format_time_timezones() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("IANA formatted time")
plot(str.format_time(1609504496000, "yyyy-MM-dd HH:mm:ssZ", "America/New_York") == "2021-01-01 07:34:56-0500" ? 1 : 0)
plot(str.format_time(1625142896000, "yyyy-MM-dd HH:mm:ssZ", "America/New_York") == "2021-07-01 08:34:56-0400" ? 1 : 0)
plot(str.format_time(1609504496000, "yyyy-MM-dd HH:mm:ssZ", "Asia/Tokyo") == "2021-01-01 21:34:56+0900" ? 1 : 0)
plot(str.format_time(1609466400000, "yyyy-MM-dd E w HH:mm:ssZ", "America/New_York") == "2020-12-31 Thu 53 21:00:00-0500" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("result");
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0]);
    }
}

#[test]
fn formats_str_format_time_short_timezone_names() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("formatted timezone names")
plot(str.format_time(1609504496000, "z zz zzz zzzz", "America/New_York") == "EST EST EST zzzz" ? 1 : 0)
plot(str.format_time(1625142896000, "z", "America/New_York") == "EDT" ? 1 : 0)
plot(str.format_time(1609504496000, "z", "Asia/Tokyo") == "JST" ? 1 : 0)
plot(str.format_time(1609459200000, "z", "UTC") == "UTC" ? 1 : 0)
plot(str.format_time(1609459200000, "z", "UTC+4") == "GMT+04:00" ? 1 : 0)
plot(str.format_time(1609459200000, "z", "-05:30") == "GMT-05:30" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("result");
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0]);
    }
}

#[test]
fn formats_datetime_literal_apostrophes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("datetime literal apostrophes")
plot(str.format_time(1609506245123, "hh 'o''clock' a ''", "UTC") == "01 o'clock PM '" ? 1 : 0)
plot(str.format("{0,time,hh 'o''clock' a ''}", 1609506245123) == "01 o'clock PM '" ? 1 : 0)
plot(str.format_time(1609506245123, "'year=''yyyy''' yyyy", "UTC") == "year='yyyy' 2021" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("result");
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0]);
    }
}

#[test]
fn formats_datetime_millisecond_widths() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("datetime millisecond widths")
plot(str.format_time(1609506245000, "S SS SSS", "UTC") == "0 00 000" ? 1 : 0)
plot(str.format_time(1609506245005, "S SS SSS", "UTC") == "5 05 005" ? 1 : 0)
plot(str.format_time(1609506245123, "S SS SSS", "UTC") == "123 123 123" ? 1 : 0)
plot(str.format("{0,time,S SS SSS}", 1609506245005) == "5 05 005" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("result");
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0]);
    }
}

#[test]
fn formats_twelve_hour_clock_boundaries() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("twelve hour clock boundaries")
plot(str.format_time(1609459200000, "h hh a", "UTC") == "0 00 AM" ? 1 : 0)
plot(str.format_time(1609502400000, "h hh a", "UTC") == "0 00 PM" ? 1 : 0)
plot(str.format_time(1609506000000, "h hh a", "UTC") == "1 01 PM" ? 1 : 0)
plot(str.format("{0,time,h hh a}", 1609459200000) == "0 00 AM" ? 1 : 0)
plot(str.format("{0,time,h hh a}", 1609502400000) == "0 00 PM" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("result");
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0]);
    }
}

#[test]
fn formats_missing_timestamps_as_unix_epoch() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("missing timestamp epoch")
plot(str.format_time(na) == "1970-01-01T00:00:00+0000" ? 1 : 0)
plot(str.format_time(na, na, na) == "1970-01-01T00:00:00+0000" ? 1 : 0)
plot(str.format_time(na, "yyyy-MM-dd HH:mm:ssZ z", "UTC-5") == "1969-12-31 19:00:00-0500 GMT-05:00" ? 1 : 0)
plot(str.format_time(na, "yyyy-MM-dd HH:mm:ssZ z", "America/New_York") == "1969-12-31 19:00:00-0500 EST" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("result");
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0]);
    }
}

#[test]
fn formats_week_of_month_within_one_to_five() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("week of month range")
plot(str.format_time(1690761600000, "W WW", "UTC") == "5 05" ? 1 : 0)
plot(str.format_time(1690855200000, "yyyy-MM-dd W", "America/New_York") == "2023-07-31 5" ? 1 : 0)
plot(str.format_time(1690855200000, "yyyy-MM-dd W", "UTC") == "2023-08-01 1" ? 1 : 0)
plot(str.format("{0,date,W WW}", 1690761600000) == "5 05" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("result");
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0]);
    }
}

#[test]
fn rejects_invalid_str_format_time_timezone() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad time")
plot(str.length(str.format_time(1609459200000, "yyyy-MM-dd", "Mars/Olympus")))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected str.format_time timezone error");

    assert!(
        error
            .message
            .contains("str.format_time unsupported timezone `Mars/Olympus`"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_invalid_substring_range() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad substring")
plot(str.length(str.substring("SMA", 2, 1)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected substring range error");

    assert!(
        error
            .message
            .contains("str.substring end_pos 1 is less than begin_pos 2"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_invalid_string_repeat_counts() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad repeat")
plot(str.length(str.repeat("x", -1)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected negative repeat error");

    assert!(
        error
            .message
            .contains("str.repeat count cannot be negative: -1"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_oversized_string_repeat_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("huge repeat")
plot(str.length(str.repeat("x", 40961)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected oversized repeat error");

    assert!(
        error
            .message
            .contains("str.repeat result cannot exceed 40960 characters"),
        "{}",
        error.message
    );
}
