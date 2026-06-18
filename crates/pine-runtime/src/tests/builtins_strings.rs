use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

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
text_percent = str.tostring(0.1234, format.percent)
text_price = str.tostring(1.234567891, format.price)
text_volume = str.tostring(1234.567, format.volume)
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
formatted_number = str.format("Rounded {0,number,#.00} Percent {1,number,percent}", 1.2, 0.0345)
formatted_integer = str.format("Integer {0,number,integer}", 1234.5)
formatted_currency = str.format("Currency {0,number,currency}", 1234.5)
formatted_array = str.format("Values {0}", values)
formatted_datetime = str.format("{0,date,yyyy-MM-dd}T{0,time,HH:mm:ssZ}", 1609459200000)
formatted_datetime_tokens = str.format("{0,date,D E w W} {0,time,HH:mm:ssZ}", 1609459200000)
formatted_quote = str.format("Literal '{0}' and apostrophe '' {0}", "X")
formatted_na = str.format(na, text_int)
formatted_na_arg = str.format("Missing {0}", na)
formatted_bool = str.format("Flag {0}", true)
match_prefix = str.match("NASDAQ:AAPL", "^(?:BATS|NASDAQ|NYSE|AMEX):")
match_suffix = str.match("NASDAQ:AAPL", "AAPL$")
match_missing = str.match("NASDAQ:AAPL", "^NYSE:")
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
plot(trimmed == upper and non_ascii_trimmed == " SMA " and na(missing_trim) and repeated == "ab-ab" and default_repeat == "abab" and empty_repeat == "" ? 1 : 0)
plot(na(missing_repeat) and na(missing_repeat_source) and na(missing_repeat_separator) ? 1 : 0)
plot(replace_first == "he1lo" and replace_second == "hel1o" and replace_missing == "hello" and replace_negative == "hello" and replace_na_occurrence == "he1lo" and replace_out_of_range == "hello" ? 1 : 0)
plot(replace_all == "he11o" and replace_all_missing == "hello" and replace_boundary == "a.b" and replace_all_boundaries == ".a.b." ? 1 : 0)
plot(na(missing_replace) and na(missing_replace_target) and na(missing_replace_replacement) and na(missing_replace_all_target) and na(missing_replace_all_replacement) ? 1 : 0)
plot(number == 1234.5 and signed_number == -0.5 and plus_number == 12 and dot_number == 0.5 ? 1 : 0)
plot(exponent_number == 1000 and signed_exponent_number == -50 and upper_exponent_number == 1.2 ? 1 : 0)
plot(na(invalid_number) and na(empty_number) and na(whitespace_number) and na(double_decimal_number) and na(bad_exponent_number) and na(nonfinite_exponent_number) and na(missing_number) ? 1 : 0)
plot(text_int == "42" and text_float == "1.25" and text_na_format == "1.25" and text_round0 == "1" and text_round1 == "1.3" ? 1 : 0)
plot(text_zeros == "1.2500" and text_percent == "12.34%" ? 1 : 0)
plot(text_price == "1.23456789" and text_volume == "1234.57" ? 1 : 0)
plot(text_bool == "true" and text_bool_false == "false" and text_string == "ok" and text_na == "NaN" ? 1 : 0)
plot(text_array == "[1, 3, NaN]" ? 1 : 0)
plot(formatted == "A=42, B=1.25, A2=42" and formatted_missing == "Missing {2}" ? 1 : 0)
plot(formatted_number == "Rounded 1.20 Percent 3.45%" and formatted_integer == "Integer 1,235" and formatted_currency == "Currency $1,234.50" ? 1 : 0)
plot(formatted_array == "Values [1.2, 2.6, NaN]" and formatted_datetime == "2021-01-01T00:00:00+0000" and formatted_datetime_tokens == "1 Fri 53 1 00:00:00+0000" and formatted_quote == "Literal {0} and apostrophe ' X" and na(formatted_na) and formatted_na_arg == "Missing NaN" and formatted_bool == "Flag true" ? 1 : 0)
plot(match_prefix == "NASDAQ:" and match_suffix == "AAPL" and match_missing == "" ? 1 : 0)
plot(na(missing_match_regex) and na(missing_match_pattern) ? 1 : 0)
plot(split_words.size() == 4 and split_words.get(0) == "A" and split_words.get(2) == "" and split_words.get(3) == "C" and split_missing_separator_literal.size() == 1 and split_missing_separator_literal.get(0) == "A,B" ? 1 : 0)
plot(split_chars.size() == 2 and split_chars.get(0) == "x" and split_chars.get(1) == "y" and split_unicode.size() == 2 and split_unicode.get(0) == "å" and split_unicode.get(1) == "β" and split_empty_source_separator.size() == 1 and split_empty_source_separator.get(0) == "" and split_empty_source_chars.size() == 0 and na(split_missing) and na(split_missing_separator) ? 1 : 0)
plot(formatted_time_default == "2021-01-01T00:00:00+0000" and formatted_time_date == "2021-01-01" and formatted_time_na_format == "2021-01-01T00:00:00+0000" ? 1 : 0)
plot(formatted_time_text == "00:00:00 on Jan 01, 2021" and na(missing_format_time) ? 1 : 0)
plot(formatted_time_alias == "00:00:00+0000" and formatted_time_gmt_alias == "00:00:00+0000" and formatted_time_na_timezone == "00:00:00+0000" ? 1 : 0)
plot(formatted_time_fixed_east == "2021-01-01 04:00:00+0400" and formatted_time_fixed_west == "2020-12-31 19:00:00-0500" and formatted_time_numeric_offset == "05:30:00+0530" ? 1 : 0)
plot(formatted_time_day_of_year == "1 01 001" and formatted_time_day_of_year_later == "33 033" ? 1 : 0)
plot(formatted_time_weekday == "Fri Friday" and formatted_time_weekday_later == "Tue Tuesday" ? 1 : 0)
plot(formatted_time_week_of_year == "53 53" and formatted_time_week_of_year_later == "5 05" ? 1 : 0)
plot(formatted_time_week_of_month == "1 01" and formatted_time_week_of_month_later == "2 02" ? 1 : 0)
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
fn rejects_unsupported_str_format_time_timezone() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad time")
plot(str.length(str.format_time(1609459200000, "yyyy-MM-dd", "America/New_York")))
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
            .contains("str.format_time unsupported timezone `America/New_York`"),
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
