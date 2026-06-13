use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn runs_float_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array ops")
values = array.new_float(2, close)
array.push(values, high)
array.set(values, 0, low)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
empty = array.new_float()
plot(first + last + array.size(values))
plot(na(missing) ? 1 : 0)
plot(na(array.pop(empty)) and array.size(empty) == 0 ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[4.0, 6.0, 8.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_float_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array methods")
values = array.new_float(2, close)
values.push(high)
values.set(0, low)
first = values.get(0)
last = values.pop()
missing = values.get(10)
empty = array.new_float()
plot(first + last + values.size())
plot(na(missing) ? 1 : 0)
plot(na(empty.pop()) and empty.size() == 0 ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[4.0, 6.0, 8.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_int_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("int array ops")
values = array.new_int(2, bar_index)
array.push(values, 10)
array.set(values, 0, 3)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot(first + last + array.size(values))
plot(na(missing) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values, &[15.0, 15.0, 15.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_array_get_with_computed_integer_index() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("computed array index")
values = array.from(10, 20, 30)
k = 2
plot(array.get(values, k - 1))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[20.0, 20.0, 20.0]);
}

#[test]
fn runs_int_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("int array methods")
values = array.new_int(2, bar_index)
values.push(10)
values.set(0, 3)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot(first + last + values.size())
plot(na(missing) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values, &[15.0, 15.0, 15.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_array_mutation_and_size_with_computed_integer_operands() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("computed array operands")
n = 1
values = array.new_float(n + 1)
array.set(values, n - 1, close)
array.set(values, n, close + 10)
plot(array.get(values, n - 1))
plot(array.get(values, n))
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[11.0, 12.0, 13.0]);
    assert_values_close(&result.plots[2].values, &[2.0, 2.0, 2.0]);
}

#[test]
fn runs_bool_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bool array ops")
values = array.new_bool(2, close > open)
array.push(values, true)
array.set(values, 0, false)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot((first or last) ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_bool_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bool array methods")
values = array.new_bool(2, close > open)
values.push(true)
values.set(0, false)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot((first or last) ? values.size() : 0)
plot(na(missing) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_string_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("string array ops")
values = array.new_string(2, "seed")
array.push(values, "tail")
array.set(values, 0, "head")
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
text = str.tostring(values)
plot(first == "head" and last == "tail" ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
plot(text == "[head, seed]" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_string_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("string array methods")
values = array.new_string(2, "seed")
values.push("tail")
values.set(0, "head")
first = values.get(0)
last = values.pop()
missing = values.get(10)
text = str.format("Values {0}", values)
plot(first == "head" and last == "tail" ? values.size() : 0)
plot(na(missing) ? 1 : 0)
plot(text == "Values [head, seed]" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_color_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("color array ops")
values = array.new_color(2, color.red)
array.push(values, color.green)
array.set(values, 0, color.blue)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot(first == color.blue and last == color.green ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
plot(color.b(first) + color.g(last))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[383.0, 383.0, 383.0]);
}

#[test]
fn runs_color_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("color array methods")
values = array.new_color(2, color.red)
values.push(color.green)
values.set(0, color.blue)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot(first == color.blue and last == color.green ? values.size() : 0)
plot(na(missing) ? 1 : 0)
plot(color.b(first) + color.g(last))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[383.0, 383.0, 383.0]);
}

#[test]
fn runs_array_clear_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array clear")
floats = array.from(close, high, na)
array.clear(floats)
array.clear(floats)
array.push(floats, low)
plot(array.size(floats))
plot(array.get(floats, 0))

ints = array.from(bar_index, 10)
ints.clear()
ints.push(7)
plot(ints.size())
plot(ints.get(0))

flags = array.from(true, false)
array.clear(flags)
flags.push(bar_index == 0)
plot(flags.size())
plot(flags.get(0) ? 1 : 0)

words = array.from("a", "b")
words.clear()
words.push("z")
plot(words.size())
plot(words.get(0) == "z" ? 1 : 0)

colors = array.from(color.red, color.green)
array.clear(colors)
colors.push(color.blue)
plot(colors.size())
plot(colors.get(0) == color.blue ? 1 : 0)

empty = array.new_float()
empty.clear()
plot(empty.size())
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 11);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[7.0, 7.0, 7.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 0.0, 0.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[8].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[9].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[10].values, &[0.0, 0.0, 0.0]);
}

#[test]
fn runs_array_helper_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array helpers")
values = array.new_int()
array.unshift(values, 2)
array.unshift(values, 1)
first = array.first(values)
last = array.last(values)
shifted = array.shift(values)
empty = array.new_string()
plot(first + last + shifted + array.size(values))
plot(array.first(values) == 2 and array.size(values) == 1 ? 1 : 0)
plot(na(array.first(empty)) and na(array.last(empty)) and na(array.shift(empty)) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[5.0, 5.0, 5.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_array_helper_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array helper methods")
values = array.new_string()
values.unshift("tail")
values.unshift("head")
first = values.first()
last = values.last()
shifted = values.shift()
colors = array.new_color()
colors.unshift(color.green)
colors.unshift(color.red)
color_first = colors.first()
color_last = colors.last()
color_shifted = colors.shift()
plot(first == "head" and last == "tail" and shifted == "head" ? values.size() : 0)
plot(color_first == color.red and color_last == color.green and color_shifted == color.red ? colors.size() : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_array_insert_remove_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array insert remove")
ints = array.new_int()
ints.push(1)
ints.push(3)
array.insert(ints, 1, 2)
removed = ints.remove(0)
plot(removed)
plot(ints.get(0) * 10 + ints.get(1))

words = array.new_string()
words.push("a")
words.push("c")
words.insert(1, "b")
word_removed = array.remove(words, 2)
plot(word_removed == "c" and words.join("|") == "a|b" ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.insert(1, color.green)
color_removed = colors.remove(0)
plot(color_removed == color.red and colors.get(0) == color.green ? 1 : 0)

flags = array.new_bool()
flags.insert(0, true)
plot(flags.remove(0) ? flags.size() : 99)

plot(na(array.remove(flags, 0)) ? 1 : 0)
array.insert(flags, 3, false)
plot(flags.size())

negative = array.from(10, 20, 30)
plot(negative.get(-1) + negative.get(-3))
negative.set(-2, 99)
plot(negative.get(1))
negative.insert(-1, 25)
plot(negative.get(2) * 100 + negative.get(-1))
negative_head = negative.remove(-4)
negative_tail = negative.remove(-1)
plot(negative_head + negative_tail + negative.size())
plot(na(negative.get(-3)) and na(negative.remove(-3)) ? 1 : 0)
plot(negative.size() == 2 ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 13);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[23.0, 23.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[7].values, &[40.0, 40.0]);
    assert_values_close(&result.plots[8].values, &[99.0, 99.0]);
    assert_values_close(&result.plots[9].values, &[2530.0, 2530.0]);
    assert_values_close(&result.plots[10].values, &[42.0, 42.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_fill_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array fill")
ints = array.new_int(4, 1)
array.fill(ints, 9, 1, 3)
plot(ints.get(0) * 1000 + ints.get(1) * 100 + ints.get(2) * 10 + ints.get(3))
ints.fill(2)
plot(ints.get(0) + ints.get(3))

floats = array.new_float(3, close)
floats.fill(high, 0, 2)
plot(floats.get(0) + floats.get(1) + floats.get(2))

words = array.new_string(3, "a")
words.fill("b", 1, 3)
plot(words.join("|") == "a|b|b" ? 1 : 0)

colors = array.new_color(2, color.red)
colors.fill(color.green)
plot(colors.get(0) == color.green and colors.get(1) == color.green ? 1 : 0)

flags = array.new_bool(2, false)
array.fill(flags, true, 0, 1)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)

array.fill(flags, false, -1, 1)
array.fill(flags, false, 0, 3)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 7);
    assert_values_close(&result.plots[0].values, &[1991.0, 1991.0]);
    assert_values_close(&result.plots[1].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[2].values, &[10.0, 15.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_from_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array from")
ints = array.from(1, 2, 3)
plot(ints.size())
plot(ints.sum())
ints.push(4)
plot(ints.last())

floats = array.from(1, close, na)
plot(floats.get(0) + floats.get(1))
plot(na(floats.get(2)) ? 1 : 0)

flags = array.from(true, false)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)

words = array.from("a", "b")
plot(words.join("|") == "a|b" ? 1 : 0)

colors = array.from(color.red, color.green)
plot(colors.get(0) == color.red and colors.get(1) == color.green ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 8);
    assert_values_close(&result.plots[0].values, &[3.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[6.0, 6.0]);
    assert_values_close(&result.plots[2].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[3].values, &[3.0, 4.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_reference_and_copy_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array references")
first_int(values) => values.get(0)
int_size(values) => array.size(values)

source = array.new_int()
alias = source
copy = array.copy(source)
method_copy = source.copy()
array.push(alias, 1)
array.push(copy, 2)
method_copy.push(3)
plot(array.size(source))
plot(array.get(source, 0))
plot(array.size(copy))
plot(array.get(copy, 0))
plot(method_copy.size())
plot(method_copy.get(0))
plot(first_int(source) + int_size(source))
plot(first_int(copy) + int_size(copy))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 8);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[3.0, 3.0, 3.0]);
    assert_values_close(&result.plots[6].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[7].values, &[3.0, 3.0, 3.0]);
}

#[test]
fn runs_varip_array_with_var_like_historical_state() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("varip arrays")
varip values = array.new_int()
values.push(1)

varip alias = values
alias.push(20)
plot(values.size())
plot(alias.size())

varip copy = array.copy(values)
copy.push(10)
plot(copy.size())
plot(values.size())

branch_out = close - close
if close >= 3
    varip branch = array.new_int()
    branch.push(1)
    branch_out := branch.size()
plot(branch_out)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 5);
    assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0, 8.0]);
    assert_values_close(&result.plots[1].values, &[2.0, 4.0, 6.0, 8.0]);
    assert_values_close(&result.plots[2].values, &[3.0, 4.0, 5.0, 6.0]);
    assert_values_close(&result.plots[3].values, &[2.0, 4.0, 6.0, 8.0]);
    assert_values_close(&result.plots[4].values, &[0.0, 0.0, 1.0, 2.0]);
}

#[test]
fn runs_array_search_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array search")
numbers = array.new_int()
array.push(numbers, 2)
array.push(numbers, 3)
array.push(numbers, 2)
plot(array.includes(numbers, 2) ? 1 : 0)
plot(array.indexof(numbers, 2))
plot(array.lastindexof(numbers, 2))
plot(numbers.indexof(9))
array.sort(numbers)
plot(array.binary_search(numbers, 2))
plot(numbers.binary_search(9))
plot(array.binary_search_leftmost(numbers, 4))
plot(array.binary_search_rightmost(numbers, 4))
plot(numbers.binary_search_leftmost(2))
plot(numbers.binary_search_rightmost(2))
plot(array.binary_search_leftmost(numbers, 1) == 0 and array.binary_search_rightmost(numbers, 1) == 0 and array.binary_search_leftmost(numbers, 9) == 2 and array.binary_search_rightmost(numbers, 9) == 2 ? 1 : 0)

truth_flags = array.from(true, true)
plot(array.every(truth_flags) and truth_flags.some() ? 1 : 0)
truth_flags.push(false)
plot(array.every(truth_flags) ? 99 : (array.some(truth_flags) ? 1 : 0))
truth_numbers = array.from(1, -2, 3)
plot(truth_numbers.every() and array.some(truth_numbers) ? 1 : 0)
truth_numbers.push(0)
plot(array.every(truth_numbers) ? 99 : 1)
truth_floats = array.new_float()
truth_floats.push(na)
truth_floats.push(0)
truth_floats.push(close)
plot(array.every(truth_floats) ? 99 : (truth_floats.some() ? 1 : 0))
empty_truth = array.new_bool()
plot(array.every(empty_truth) and not empty_truth.some() ? 1 : 0)
na_truth = array.new_int(2)
plot(array.every(na_truth) ? 99 : (array.some(na_truth) ? 98 : 1))

words = array.new_string()
words.push("a")
words.push("b")
words.push("a")
plot(words.includes("b") ? words.lastindexof("a") : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
plot(colors.includes(color.green) ? colors.indexof(color.green) : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 20);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[2].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[3].values, &[-1.0, -1.0, -1.0]);
    assert_values_close(&result.plots[4].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[5].values, &[-1.0, -1.0, -1.0]);
    assert_values_close(&result.plots[6].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[7].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[8].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[9].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[10].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[12].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[14].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[15].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[16].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[17].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[18].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[19].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_numeric_array_statistics() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array statistics")
ints = array.new_int()
array.push(ints, 2)
array.push(ints, 5)
array.push(ints, 1)
plot(array.min(ints))
plot(array.max(ints))
plot(array.sum(ints))
plot(array.avg(ints))
plot(array.range(ints))
plot(array.median(ints))
plot(array.percentile_nearest_rank(ints, 50))
plot(ints.percentile_linear_interpolation(75))
plot(array.percentrank(ints, 1))
plot(array.variance(ints, false))
mode_ints = array.from(1, 3, 3, 2, 2)
plot(mode_ints.mode())

floats = array.new_float()
floats.push(close)
floats.push(high)
floats.push(na)
plot(floats.min())
plot(floats.max())
plot(floats.sum())
plot(floats.avg())
plot(floats.range())
plot(floats.median())
plot(floats.percentile_nearest_rank(50))
plot(array.percentile_linear_interpolation(floats, 50))
plot(floats.percentrank(1))
plot(array.variance(floats))
plot(floats.stdev(false))

signs = array.from(-2, 0, 3)
absolutes = signs.abs()
plot(absolutes.get(0) + absolutes.get(1) + absolutes.get(2))
plot(signs.get(0))
float_signs = array.new_float()
float_signs.push(-close)
float_signs.push(na)
float_abs = array.abs(float_signs)
plot(float_abs.get(0))
plot(na(float_abs.get(1)) ? 1 : 0)

standard_values = array.from(2, 4, 4, 4, 5, 5, 7, 9)
standardized = standard_values.standardize()
plot(standardized.get(0))
plot(standardized.get(7))
plot(standard_values.get(0))
standard_with_na = array.from(close, na, high)
standardized_with_na = array.standardize(standard_with_na)
plot(standardized_with_na.size())
plot(na(standardized_with_na.get(1)) ? 1 : 0)

covariance_x = array.from(1, 2, 3)
covariance_y = array.from(1, 5, 7)
plot(array.covariance(covariance_x, covariance_y))
plot(covariance_x.covariance(covariance_y, false))
covariance_with_na_x = array.from(close, na, high)
covariance_with_na_y = array.from(open, close, na)
plot(array.covariance(covariance_with_na_x, covariance_with_na_y))
plot(na(covariance_with_na_x.covariance(covariance_with_na_y, false)) ? 1 : 0)
mismatched_covariance = array.from(1, 2)
plot(na(array.covariance(covariance_x, mismatched_covariance)) ? 1 : 0)

empty = array.new_float()
only_na = array.new_int(2)
empty_standardized = array.standardize(empty)
only_na_standardized = only_na.standardize()
plot(na(array.min(empty)) and na(array.max(only_na)) and na(array.sum(empty)) and na(array.avg(only_na)) and na(array.range(empty)) and na(array.mode(ints)) and na(array.percentile_nearest_rank(empty, 50)) and na(array.percentile_linear_interpolation(ints, 150)) and na(array.percentrank(empty, 0)) and empty_standardized.size() == 0 and only_na_standardized.size() == 0 and na(array.covariance(empty, empty)) and na(array.variance(empty)) and na(only_na.stdev()) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 37);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[5.0, 5.0]);
    assert_values_close(&result.plots[2].values, &[8.0, 8.0]);
    assert_values_close(&result.plots[3].values, &[8.0 / 3.0, 8.0 / 3.0]);
    assert_values_close(&result.plots[4].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[5].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[6].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[7].values, &[3.5, 3.5]);
    assert_values_close(&result.plots[8].values, &[100.0, 100.0]);
    assert_values_close(&result.plots[9].values, &[13.0 / 3.0, 13.0 / 3.0]);
    assert_values_close(&result.plots[10].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[11].values, &[2.0, 3.0]);
    assert_values_close(&result.plots[12].values, &[4.0, 6.0]);
    assert_values_close(&result.plots[13].values, &[6.0, 9.0]);
    assert_values_close(&result.plots[14].values, &[3.0, 4.5]);
    assert_values_close(&result.plots[15].values, &[2.0, 3.0]);
    assert_values_close(&result.plots[16].values, &[3.0, 4.5]);
    assert_values_close(&result.plots[17].values, &[2.0, 3.0]);
    assert_values_close(&result.plots[18].values, &[3.0, 4.5]);
    assert_values_close(&result.plots[19].values, &[100.0, 100.0]);
    assert_values_close(&result.plots[20].values, &[1.0, 2.25]);
    assert_values_close(&result.plots[21].values, &[2.0_f64.sqrt(), 4.5_f64.sqrt()]);
    assert_values_close(&result.plots[22].values, &[5.0, 5.0]);
    assert_values_close(&result.plots[23].values, &[-2.0, -2.0]);
    assert_values_close(&result.plots[24].values, &[2.0, 3.0]);
    assert_values_close(&result.plots[25].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[26].values, &[-1.5, -1.5]);
    assert_values_close(&result.plots[27].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[28].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[29].values, &[3.0, 3.0]);
    assert_values_close(&result.plots[30].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[31].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[32].values, &[3.0, 3.0]);
    assert_values_close(&result.plots[33].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[34].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[35].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[36].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_ordering_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array ordering")
ints = array.new_int()
array.push(ints, 3)
array.push(ints, 1)
array.push(ints, 2)
array.sort(ints)
plot(ints.get(0) * 100 + ints.get(1) * 10 + ints.get(2))
desc_ints = array.from(1, 3, 2)
desc_ints.sort(order.descending)
plot(desc_ints.get(0) * 100 + desc_ints.get(1) * 10 + desc_ints.get(2))
desc_float_special = array.new_float()
desc_float_special.push(na)
desc_float_special.push(close)
desc_float_special.push(high)
desc_float_special.sort(order.descending)
plot(na(desc_float_special.get(0)) and desc_float_special.get(1) == high and desc_float_special.get(2) == close ? 1 : 0)
ints.reverse()
plot(ints.get(0) * 100 + ints.get(1) * 10 + ints.get(2))
unsorted_ints = array.from(30, 10, 20)
sorted_int_indices = unsorted_ints.sort_indices()
plot(sorted_int_indices.get(0) * 100 + sorted_int_indices.get(1) * 10 + sorted_int_indices.get(2))
desc_sorted_int_indices = unsorted_ints.sort_indices(order.descending)
plot(desc_sorted_int_indices.get(0) * 100 + desc_sorted_int_indices.get(1) * 10 + desc_sorted_int_indices.get(2))
plot(unsorted_ints.get(0) * 100 + unsorted_ints.get(1) * 10 + unsorted_ints.get(2))

floats = array.new_float()
floats.push(na)
floats.push(high)
floats.push(close)
floats.sort()
plot(floats.get(0) + floats.get(1))
plot(na(floats.get(2)) ? 1 : 0)
float_indices_source = array.new_float()
float_indices_source.push(na)
float_indices_source.push(high)
float_indices_source.push(close)
float_indices = array.sort_indices(float_indices_source)
plot(float_indices.get(0) * 100 + float_indices.get(1) * 10 + float_indices.get(2))
array.reverse(floats)
plot(na(floats.get(0)) and floats.get(1) == high and floats.get(2) == close ? 1 : 0)

words = array.new_string()
words.push("b")
words.push("a")
words.push("c")
words.push("")
array.sort(words)
plot(words.get(0) == "a" and words.get(1) == "b" and words.get(2) == "c" and words.get(3) == "" ? 1 : 0)
words.sort(order.descending)
plot(words.get(0) == "" and words.get(1) == "c" and words.get(2) == "b" and words.get(3) == "a" ? 1 : 0)
word_indices = words.sort_indices(order.ascending)
plot(word_indices.get(0) == 3 and word_indices.get(1) == 2 and word_indices.get(2) == 1 and word_indices.get(3) == 0 ? 1 : 0)
words.reverse()
plot(words.get(0) == "a" and words.get(1) == "b" and words.get(2) == "c" and words.get(3) == "" ? 1 : 0)

flags = array.from(true, false, false)
flags.reverse()
plot(not flags.get(0) and not flags.get(1) and flags.get(2) ? 1 : 0)

empty_sort = array.new_float()
array.sort(empty_sort)
plot(empty_sort.size())
empty_sort_indices = empty_sort.sort_indices()
plot(empty_sort_indices.size())
empty_sort.reverse()
plot(empty_sort.size())

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
colors.reverse()
plot(colors.get(0) == color.green and colors.get(1) == color.red ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 20);
    assert_values_close(&result.plots[0].values, &[123.0, 123.0]);
    assert_values_close(&result.plots[1].values, &[321.0, 321.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[321.0, 321.0]);
    assert_values_close(&result.plots[4].values, &[120.0, 120.0]);
    assert_values_close(&result.plots[5].values, &[21.0, 21.0]);
    assert_values_close(&result.plots[6].values, &[3120.0, 3120.0]);
    assert_values_close(&result.plots[7].values, &[6.0, 9.0]);
    assert_values_close(&result.plots[8].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[9].values, &[210.0, 210.0]);
    assert_values_close(&result.plots[10].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[14].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[15].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[16].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[17].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[18].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[19].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_join_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array join")
ints = array.new_int()
ints.push(1)
ints.push(2)
plot(array.join(ints, "|") == "1|2" ? 1 : 0)

floats = array.new_float()
floats.push(1.25)
floats.push(2.5)
plot(floats.join() == "1.25,2.5" ? 1 : 0)
floats.push(na)
plot(array.join(floats, "|") == "1.25|2.5|NaN" ? 1 : 0)

flags = array.new_bool()
flags.push(false)
flags.push(true)
plot(array.join(flags, "/") == "false/true" ? 1 : 0)

words = array.new_string()
words.push("a")
words.push("b")
plot(words.join("-") == "a-b" ? 1 : 0)
plot(words.join(na) == "a,b" ? 1 : 0)
words.insert(1, "")
plot(words.join("|") == "a||b" ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
plot(colors.join("|") == "16711680|32768" ? 1 : 0)

empty = array.new_string()
plot(array.join(empty, "|") == "" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 9);
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0, 1.0]);
    }
}

#[test]
fn rejects_oversized_array_join_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array join limit")
values = array.new_string(410)
array.set(values, 0, str.repeat("x", 100))
plot(str.length(array.join(values, str.repeat("y", 100))))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array.join limit error");

    assert!(
        error
            .message
            .contains("array.join result cannot exceed 40960 characters"),
        "{}",
        error.message
    );
}

#[test]
fn runs_array_slice_concat_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array slice concat")
ints = array.new_int()
ints.push(1)
ints.push(2)
ints.push(3)
part = array.slice(ints, 1, 3)
part.set(0, 20)
plot(part.size())
plot(part.get(0) + part.get(1))
plot(ints.get(1))

more = array.new_int()
more.push(4)
returned = array.concat(ints, more)
plot(array.size(ints))
plot(array.size(returned))
plot(returned.get(3))

words = array.new_string()
words.push("a")
words.push("b")
words.push("c")
tail = words.slice(1, 3)
extra = array.new_string()
extra.push("d")
words.concat(extra)
plot(tail.join("|") == "b|c" and words.join("|") == "a|b|c|d" ? 1 : 0)
empty_extra = array.new_string()
words.concat(empty_extra)
plot(words.join("|") == "a|b|c|d" and empty_extra.size() == 0 ? 1 : 0)
empty_target = array.new_string()
empty_target.concat(extra)
plot(empty_target.join("|") == "d" and extra.size() == 1 ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
colors_tail = colors.slice(1, 2)
colors.concat(colors_tail)
plot(colors.get(2) == color.green ? 1 : 0)

floats = array.new_float()
floats.push(close)
floats.push(na)
floats.push(high)
float_head = array.slice(floats, 0, 2)
plot(float_head.size())
plot(float_head.get(0) == close and na(float_head.get(1)) ? 1 : 0)
float_more = array.from(na, high)
float_returned = floats.concat(float_more)
plot(float_returned.size() == 5 and floats.size() == 5 and na(floats.get(3)) and floats.get(4) == high and float_more.size() == 2 ? 1 : 0)

flags = array.from(true, false, true)
flag_tail = flags.slice(1, 3)
flag_tail.set(0, true)
plot(flag_tail.size())
plot(flag_tail.get(0) and flag_tail.get(1) and not flags.get(1) ? 1 : 0)
bool_more = array.from(false)
flags.concat(bool_more)
plot(flags.size() == 4 and not flags.get(3) and bool_more.size() == 1 ? 1 : 0)

empty_window = ints.slice(2, 2)
plot(empty_window.size())
plot(na(array.slice(ints, -1, 1)) and na(ints.slice(1, 5)) and na(array.slice(ints, 2, 1)) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 18);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[23.0, 23.0]);
    assert_values_close(&result.plots[2].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[3].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[4].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[5].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[8].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[9].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[10].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[14].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[15].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[16].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[17].values, &[1.0, 1.0]);
}

#[test]
fn handles_invalid_array_slice_bounds() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array slice bounds")
values = array.new_int()
values.push(1)
plot(na(array.slice(values, -1, 1)) ? 1 : 0)
plot(na(values.slice(1, 3)) ? 1 : 0)
plot(na(array.slice(values, 1, 0)) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[1.0]);
    assert_values_close(&result.plots[1].values, &[1.0]);
    assert_values_close(&result.plots[2].values, &[1.0]);
}

#[test]
fn rejects_oversized_array_concat_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array concat limit")
left = array.new_int(100000, 1)
right = array.new_int(1, 2)
array.concat(left, right)
plot(array.size(left))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array.concat limit error");

    assert!(
        error
            .message
            .contains("array.concat cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_oversized_array_insert_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array insert limit")
values = array.new_int(100000, 1)
array.insert(values, 0, 2)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array.insert limit error");

    assert!(
        error
            .message
            .contains("array.insert cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn var_float_array_persists_across_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("var array")
var values = array.new_float()
fresh = array.new_float()
array.push(values, close)
array.push(fresh, close)
plot(array.size(values))
plot(array.size(fresh))
plot(array.get(values, 0))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn handles_float_array_edge_cases() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array edges")
values = array.new_float()
missing = array.get(values, 0)
popped = array.pop(values)
array.set(values, 10, close)
plot(na(missing) ? 1 : 0)
plot(na(popped) ? 1 : 0)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[1.0]);
    assert_values_close(&result.plots[1].values, &[1.0]);
    assert_values_close(&result.plots[2].values, &[0.0]);
}

#[test]
fn rejects_negative_float_array_size() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array negative size")
values = array.new_float(-1)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected negative array size error");

    assert!(
        error
            .message
            .contains("array.new_float size cannot be negative"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_oversized_float_array_creation() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array oversized")
values = array.new_float(100001)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected oversized array error");

    assert!(
        error
            .message
            .contains("array.new_float size cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_float_array_push_past_limit() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array push limit")
values = array.new_float(100000)
array.push(values, close)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array push limit error");

    assert!(
        error
            .message
            .contains("array.push cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_float_array_unshift_past_limit() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array unshift limit")
values = array.new_float(100000)
array.unshift(values, close)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array unshift limit error");

    assert!(
        error
            .message
            .contains("array.unshift cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn profiles_float_array_storage() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array profile")
var values = array.new_float()
array.push(values, close)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let profiled = run_historical_profiled(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0)])
        .expect("profiled runtime result");

    assert_eq!(profiled.profile.array_slots, 1);
    assert_eq!(profiled.profile.array_values, 2);
    assert!(profiled.profile.array_value_capacity >= 2);
}

#[test]
fn runs_readonly_float_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array udf")
first(values) => array.get(values, 0)
var values = array.new_float()
array.push(values, close)
plot(first(values) + array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 3.0, 4.0]);
}

#[test]
fn runs_readonly_int_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("int array udf")
first(values) => array.get(values, 0)
var values = array.new_int()
array.push(values, bar_index)
plot(first(values) + array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
}

#[test]
fn runs_readonly_bool_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bool array udf")
first(values) => array.get(values, 0)
var values = array.new_bool()
array.push(values, bar_index == 0)
plot(first(values) ? array.size(values) : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
}

#[test]
fn runs_readonly_string_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("string array udf")
first(values) => array.get(values, 0)
var values = array.new_string()
array.push(values, "seed")
plot(first(values) == "seed" ? array.size(values) : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
}

#[test]
fn runs_readonly_color_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("color array udf")
first(values) => array.get(values, 0)
var values = array.new_color()
array.push(values, color.red)
plot(first(values) == color.red ? array.size(values) : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
}
