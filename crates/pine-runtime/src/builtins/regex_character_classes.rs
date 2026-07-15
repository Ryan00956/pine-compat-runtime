use std::{cmp, ops::Range};

use regex_syntax::{
    ParserBuilder,
    ast::{self, Ast, ClassBracketed, ClassSet, ClassSetBinaryOpKind, ClassSetItem, Span},
    hir::{Class, ClassUnicode, ClassUnicodeRange, HirKind},
};

use super::regex_escapes::is_pine_regex_line_separator;

pub(crate) fn pine_posix_class(name: &str, unicode: bool, negated: bool) -> Option<&'static str> {
    let (positive, negative) = if unicode {
        match name.to_ascii_uppercase().as_str() {
            "LOWER" => (r"\p{Lowercase}", r"\P{Lowercase}"),
            "UPPER" => (r"\p{Uppercase}", r"\P{Uppercase}"),
            "ASCII" => (r"[\x00-\x7F]", r"[^\x00-\x7F]"),
            "ALPHA" => (r"\p{Alphabetic}", r"\P{Alphabetic}"),
            "DIGIT" => (r"\p{Nd}", r"\P{Nd}"),
            "ALNUM" => (r"[\p{Alphabetic}\p{Nd}]", r"[^\p{Alphabetic}\p{Nd}]"),
            "PUNCT" => (r"\p{Punctuation}", r"\P{Punctuation}"),
            "GRAPH" => (
                r"[^\p{White_Space}\p{Cc}\p{Cn}]",
                r"[\p{White_Space}\p{Cc}\p{Cn}]",
            ),
            "PRINT" => (
                r"[[^\p{White_Space}\p{Cc}\p{Cn}]\p{Zs}]",
                r"[[\p{White_Space}\p{Cc}\p{Cn}]&&[^\p{Zs}]]",
            ),
            "BLANK" => (r"[\p{Zs}\t]", r"[^\p{Zs}\t]"),
            "CNTRL" => (r"\p{Cc}", r"\P{Cc}"),
            "XDIGIT" => (r"[\p{Nd}\p{Hex_Digit}]", r"[^\p{Nd}\p{Hex_Digit}]"),
            "SPACE" => (r"\p{White_Space}", r"\P{White_Space}"),
            _ => return None,
        }
    } else {
        match name {
            "Lower" => (r"[a-z]", r"[^a-z]"),
            "Upper" => (r"[A-Z]", r"[^A-Z]"),
            "ASCII" => (r"[\x00-\x7F]", r"[^\x00-\x7F]"),
            "Alpha" => (r"[A-Za-z]", r"[^A-Za-z]"),
            "Digit" => (r"[0-9]", r"[^0-9]"),
            "Alnum" => (r"[A-Za-z0-9]", r"[^A-Za-z0-9]"),
            "Punct" => (
                r"[\x21-\x2F\x3A-\x40\x5B-\x60\x7B-\x7E]",
                r"[^\x21-\x2F\x3A-\x40\x5B-\x60\x7B-\x7E]",
            ),
            "Graph" => (r"[\x21-\x7E]", r"[^\x21-\x7E]"),
            "Print" => (r"[\x20-\x7E]", r"[^\x20-\x7E]"),
            "Blank" => (r"[\x20\t]", r"[^\x20\t]"),
            "Cntrl" => (r"[\x00-\x1F\x7F]", r"[^\x00-\x1F\x7F]"),
            "XDigit" => (r"[0-9A-Fa-f]", r"[^0-9A-Fa-f]"),
            "Space" => (r"[\x20\t\n\x0B\f\r]", r"[^\x20\t\n\x0B\f\r]"),
            _ => return None,
        }
    };
    Some(if negated { negative } else { positive })
}

pub(crate) fn pine_unicode_block_property(property: &str) -> Option<&str> {
    if let Some(name) = property.strip_prefix("In").filter(|name| !name.is_empty()) {
        return Some(name);
    }
    let (kind, name) = property.split_once('=')?;
    (kind.eq_ignore_ascii_case("Block") && !name.is_empty()).then_some(name)
}

pub(crate) fn push_pine_unicode_block(result: &mut String, start: u32, end: u32, negated: bool) {
    if start >= 0xD800 && end <= 0xDFFF {
        result.push_str(if negated {
            r"[\x{0}-\x{D7FF}\x{E000}-\x{10FFFF}]"
        } else {
            r"[a&&b]"
        });
        return;
    }

    result.push_str(if negated { "[^" } else { "[" });
    use std::fmt::Write;
    write!(result, r"\x{{{start:X}}}-\x{{{end:X}}}]").expect("writing to a String cannot fail");
}

#[derive(Clone, Copy, Default)]
enum PineRegexClassRangeState {
    #[default]
    NoStart,
    Single,
    Endpoint,
}

pub(crate) struct PineRegexClassPrefix {
    pub(crate) can_negate: bool,
    pub(crate) can_literal_close: bool,
    range: PineRegexClassRangeState,
    has_atom: bool,
    output_start: usize,
}

impl PineRegexClassPrefix {
    pub(crate) fn new(output_start: usize) -> Self {
        Self {
            can_negate: true,
            can_literal_close: true,
            range: PineRegexClassRangeState::NoStart,
            has_atom: false,
            output_start,
        }
    }

    pub(crate) fn mark_atom(&mut self) {
        self.can_negate = false;
        self.can_literal_close = false;
        self.has_atom = true;
    }

    pub(crate) fn mark_scalar(&mut self) {
        self.mark_atom();
        self.range = match self.range {
            PineRegexClassRangeState::Endpoint => PineRegexClassRangeState::NoStart,
            _ => PineRegexClassRangeState::Single,
        };
    }

    pub(crate) fn mark_set(&mut self) -> bool {
        self.mark_atom();
        let invalid_endpoint = matches!(self.range, PineRegexClassRangeState::Endpoint);
        self.range = PineRegexClassRangeState::NoStart;
        invalid_endpoint
    }

    pub(crate) fn expects_range_endpoint(&self) -> bool {
        matches!(self.range, PineRegexClassRangeState::Endpoint)
    }

    pub(crate) fn mark_raw_hyphen(&mut self, immediate_next: Option<u8>) -> bool {
        self.mark_atom();
        if matches!(self.range, PineRegexClassRangeState::Endpoint) {
            self.range = PineRegexClassRangeState::NoStart;
            return false;
        }
        if matches!(self.range, PineRegexClassRangeState::Single)
            && !matches!(immediate_next, Some(b'[' | b']'))
        {
            self.range = PineRegexClassRangeState::Endpoint;
            return true;
        }
        self.range = PineRegexClassRangeState::Single;
        false
    }

    pub(crate) fn mark_intersection(&mut self) {
        self.mark_atom();
        self.range = PineRegexClassRangeState::NoStart;
    }

    pub(crate) fn has_atom(&self) -> bool {
        self.has_atom
    }

    pub(crate) fn output_start(&self) -> usize {
        self.output_start
    }
}

// Java's parser can carry a mutable BitClass through later intersections. The
// flag limits grouping rewrites to prefixes whose membership cannot be revived.
pub(crate) enum PineRegexEmptyIntersection {
    Redundant { can_group: bool },
    Repeat { current: String, can_group: bool },
    Invalid,
}

impl PineRegexEmptyIntersection {
    pub(crate) fn can_group(&self) -> bool {
        match self {
            Self::Redundant { can_group } | Self::Repeat { can_group, .. } => *can_group,
            Self::Invalid => false,
        }
    }
}

fn is_java_bit_class_literal(ch: char, unicode_case_insensitive: bool) -> bool {
    ch <= '\u{00FF}'
        && !(unicode_case_insensitive
            && matches!(
                ch,
                '\u{00FF}' | '\u{00B5}' | 'I' | 'i' | 'S' | 's' | 'K' | 'k' | 'Å' | 'å'
            ))
}

fn java_union_item_is_bit_class(item: &ClassSetItem, unicode_case_insensitive: bool) -> bool {
    matches!(item, ClassSetItem::Literal(literal) if is_java_bit_class_literal(literal.c, unicode_case_insensitive))
}

pub(crate) fn pine_java_empty_intersection(
    class_prefix: &str,
    verbose: bool,
    unicode_case_insensitive: bool,
) -> PineRegexEmptyIntersection {
    let pattern = format!("{class_prefix}]");
    let mut builder = ast::parse::ParserBuilder::new();
    builder.ignore_whitespace(verbose);
    let Ok(Ast::ClassBracketed(ref class)) = builder.build().parse(&pattern) else {
        return PineRegexEmptyIntersection::Invalid;
    };

    let ClassSet::Item(item) = &class.kind else {
        return PineRegexEmptyIntersection::Redundant { can_group: false };
    };
    let ClassSetItem::Union(union) = item else {
        return PineRegexEmptyIntersection::Redundant { can_group: true };
    };
    let Some(last) = union.items.last() else {
        return PineRegexEmptyIntersection::Invalid;
    };
    if union
        .items
        .iter()
        .all(|item| java_union_item_is_bit_class(item, unicode_case_insensitive))
    {
        return PineRegexEmptyIntersection::Redundant { can_group: true };
    }
    if java_union_item_is_bit_class(last, unicode_case_insensitive) {
        return PineRegexEmptyIntersection::Invalid;
    }

    let span = last.span();
    pattern
        .get(span.start.offset..span.end.offset)
        .map(str::to_owned)
        .map(|current| PineRegexEmptyIntersection::Repeat {
            current,
            can_group: false,
        })
        .unwrap_or(PineRegexEmptyIntersection::Invalid)
}

pub(crate) fn open_pine_regex_class_group(
    result: &mut String,
    class_start: usize,
    protected_spans: &mut [Range<usize>],
) {
    let mut insert_at = class_start + 1;
    if result.as_bytes().get(insert_at) == Some(&b'^') {
        insert_at += 1;
    }
    result.insert(insert_at, '[');
    for span in protected_spans {
        if span.start >= insert_at {
            span.start += 1;
            span.end += 1;
        }
    }
}

fn is_verbose_ascii_space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\u{000b}' | '\u{000c}' | '\r')
}

pub(crate) fn next_pine_regex_class_token(
    pattern: &str,
    mut index: usize,
    verbose: bool,
) -> Option<(usize, char)> {
    loop {
        let token_start = index;
        let ch = pattern.get(index..)?.chars().next()?;
        index += ch.len_utf8();
        if !verbose || (!is_verbose_ascii_space(ch) && ch != '#') {
            return Some((token_start, ch));
        }
        if is_verbose_ascii_space(ch) {
            continue;
        }

        while let Some(comment_ch) = pattern.get(index..)?.chars().next() {
            let comment_start = index;
            index += comment_ch.len_utf8();
            if matches!(comment_ch, '\n' | '\r') {
                break;
            }
            if is_pine_regex_line_separator(comment_ch) {
                return Some((comment_start, comment_ch));
            }
        }
    }
}

#[derive(Clone, Copy)]
enum CaseFoldMode {
    Ascii,
    Unicode,
}

fn span_is_protected(span: &Span, protected: &[Range<usize>]) -> bool {
    protected
        .iter()
        .any(|range| range.start <= span.start.offset && span.end.offset <= range.end)
}

fn class_from_hir_pattern(pattern: &str) -> Option<ClassUnicode> {
    let mut parser = ParserBuilder::new().build();
    let hir = parser.parse(pattern).ok()?;
    match hir.kind() {
        HirKind::Empty => Some(ClassUnicode::empty()),
        HirKind::Literal(literal) => {
            let text = std::str::from_utf8(&literal.0).ok()?;
            let mut chars = text.chars();
            let ch = chars.next()?;
            chars
                .next()
                .is_none()
                .then(|| ClassUnicode::new([ClassUnicodeRange::new(ch, ch)]))
        }
        HirKind::Class(Class::Unicode(class)) => Some(class.clone()),
        HirKind::Class(Class::Bytes(class)) => class.to_unicode_class(),
        _ => None,
    }
}

fn class_from_span(pattern: &str, span: &Span) -> Option<ClassUnicode> {
    let item = pattern.get(span.start.offset..span.end.offset)?;
    class_from_hir_pattern(&format!("[{item}]"))
}

fn fold_ascii(mut class: ClassUnicode) -> ClassUnicode {
    let mut additions = Vec::new();
    for range in class.ranges() {
        let lower_start = cmp::max(range.start(), 'a');
        let lower_end = cmp::min(range.end(), 'z');
        if lower_start <= lower_end {
            additions.push(ClassUnicodeRange::new(
                lower_start.to_ascii_uppercase(),
                lower_end.to_ascii_uppercase(),
            ));
        }

        let upper_start = cmp::max(range.start(), 'A');
        let upper_end = cmp::min(range.end(), 'Z');
        if upper_start <= upper_end {
            additions.push(ClassUnicodeRange::new(
                upper_start.to_ascii_lowercase(),
                upper_end.to_ascii_lowercase(),
            ));
        }
    }
    for range in additions {
        class.push(range);
    }
    class
}

fn fold_class(mut class: ClassUnicode, mode: CaseFoldMode) -> Option<ClassUnicode> {
    match mode {
        CaseFoldMode::Ascii => Some(fold_ascii(class)),
        CaseFoldMode::Unicode => {
            class.try_case_fold_simple().ok()?;
            Some(class)
        }
    }
}

fn fold_translated_item(
    pattern: &str,
    span: &Span,
    negated: bool,
    mode: CaseFoldMode,
) -> Option<ClassUnicode> {
    let mut class = class_from_span(pattern, span)?;
    if negated {
        class.negate();
    }
    class = fold_class(class, mode)?;
    if negated {
        class.negate();
    }
    Some(class)
}

fn evaluate_item(
    pattern: &str,
    item: &ClassSetItem,
    mode: CaseFoldMode,
    protected: &[Range<usize>],
) -> Option<ClassUnicode> {
    if span_is_protected(item.span(), protected) {
        return class_from_span(pattern, item.span());
    }

    match item {
        ClassSetItem::Empty(_) => Some(ClassUnicode::empty()),
        ClassSetItem::Literal(literal) => fold_class(
            ClassUnicode::new([ClassUnicodeRange::new(literal.c, literal.c)]),
            mode,
        ),
        ClassSetItem::Range(range) => fold_class(
            ClassUnicode::new([ClassUnicodeRange::new(range.start.c, range.end.c)]),
            mode,
        ),
        ClassSetItem::Ascii(class) => {
            fold_translated_item(pattern, &class.span, class.negated, mode)
        }
        ClassSetItem::Perl(class) => {
            fold_translated_item(pattern, &class.span, class.negated, mode)
        }
        ClassSetItem::Unicode(class) => {
            let mut translated = class_from_span(pattern, &class.span)?;
            if class.is_negated() {
                translated.negate();
            }
            translated.try_case_fold_simple().ok()?;
            if class.is_negated() {
                translated.negate();
            }
            Some(translated)
        }
        ClassSetItem::Bracketed(class) => evaluate_bracketed(pattern, class, mode, protected),
        ClassSetItem::Union(union) => {
            let mut class = ClassUnicode::empty();
            for item in &union.items {
                class.union(&evaluate_item(pattern, item, mode, protected)?);
            }
            Some(class)
        }
    }
}

fn evaluate_set(
    pattern: &str,
    set: &ClassSet,
    mode: CaseFoldMode,
    protected: &[Range<usize>],
) -> Option<ClassUnicode> {
    match set {
        ClassSet::Item(item) => evaluate_item(pattern, item, mode, protected),
        ClassSet::BinaryOp(operation) => {
            let mut lhs = evaluate_set(pattern, &operation.lhs, mode, protected)?;
            let rhs = evaluate_set(pattern, &operation.rhs, mode, protected)?;
            match operation.kind {
                ClassSetBinaryOpKind::Intersection => lhs.intersect(&rhs),
                ClassSetBinaryOpKind::Difference => lhs.difference(&rhs),
                ClassSetBinaryOpKind::SymmetricDifference => lhs.symmetric_difference(&rhs),
            }
            Some(lhs)
        }
    }
}

fn evaluate_bracketed(
    pattern: &str,
    bracketed: &ClassBracketed,
    mode: CaseFoldMode,
    protected: &[Range<usize>],
) -> Option<ClassUnicode> {
    if span_is_protected(&bracketed.span, protected) {
        return class_from_span(pattern, &bracketed.span);
    }

    let mut class = evaluate_set(pattern, &bracketed.kind, mode, protected)?;
    if bracketed.negated {
        class.negate();
    }
    Some(class)
}

fn serialize_class(class: &ClassUnicode) -> String {
    if class.ranges().is_empty() {
        return "[a&&b]".to_owned();
    }

    let mut result = String::from("[");
    for range in class.ranges() {
        use std::fmt::Write;

        write!(result, r"\x{{{:X}}}", u32::from(range.start()))
            .expect("writing to a String cannot fail");
        if range.start() != range.end() {
            write!(result, r"-\x{{{:X}}}", u32::from(range.end()))
                .expect("writing to a String cannot fail");
        }
    }
    result.push(']');
    result
}

pub(crate) fn normalize_case_insensitive_class(
    pattern: &str,
    unicode_case: bool,
    protected: &[Range<usize>],
) -> Option<String> {
    let mut parser = ast::parse::Parser::new();
    let ast = parser.parse(pattern).ok()?;
    let Ast::ClassBracketed(ref bracketed) = ast else {
        return None;
    };
    let mode = if unicode_case {
        CaseFoldMode::Unicode
    } else {
        CaseFoldMode::Ascii
    };
    Some(serialize_class(&evaluate_bracketed(
        pattern, bracketed, mode, protected,
    )?))
}

pub(crate) fn push_case_folded_class(result: &mut String, class: &str) {
    result.push_str("(?-i:");
    result.push_str(class);
    result.push(')');
}

pub(crate) enum PineRegexClassReplacementCase {
    Fold {
        replacement_unicode: bool,
        outer_unicode: bool,
    },
    Exact,
}

pub(crate) fn push_case_insensitive_class_replacement(
    result: &mut String,
    replacement: &str,
    case_insensitive: bool,
    replacement_case: PineRegexClassReplacementCase,
    class_depth: usize,
    protected_spans: &mut Vec<Range<usize>>,
) {
    if !case_insensitive {
        result.push_str(replacement);
        return;
    }

    let (replacement_unicode, outer_unicode, exact) = match replacement_case {
        PineRegexClassReplacementCase::Fold {
            replacement_unicode,
            outer_unicode,
        } => (replacement_unicode, outer_unicode, false),
        PineRegexClassReplacementCase::Exact => (false, false, true),
    };

    if class_depth > 0 {
        let start = result.len();
        if exact {
            result.push_str(replacement);
            protected_spans.push(start..result.len());
        } else if replacement_unicode != outer_unicode
            && let Some(class) =
                normalize_case_insensitive_class(replacement, replacement_unicode, &[])
        {
            result.push_str(&class);
            protected_spans.push(start..result.len());
        } else {
            result.push_str(replacement);
        }
        return;
    }

    if exact {
        push_case_folded_class(result, replacement);
    } else if let Some(class) =
        normalize_case_insensitive_class(replacement, replacement_unicode, &[])
    {
        push_case_folded_class(result, &class);
    } else {
        result.push_str(replacement);
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::normalize_case_insensitive_class;

    fn matcher(pattern: &str, unicode_case: bool, protected: &[std::ops::Range<usize>]) -> Regex {
        let class = normalize_case_insensitive_class(pattern, unicode_case, protected)
            .expect("normalized case-insensitive class");
        Regex::new(&format!("(?-i:\\A{class}\\z)")).expect("serialized class compiles")
    }

    #[test]
    fn folds_ascii_and_unicode_literals_before_class_operations() {
        let ascii = matcher("[kβ]", false, &[]);
        assert!(ascii.is_match("K"));
        assert!(ascii.is_match("β"));
        assert!(!ascii.is_match("K"));
        assert!(!ascii.is_match("Β"));

        let unicode = matcher("[kβ]", true, &[]);
        assert!(unicode.is_match("K"));
        assert!(unicode.is_match("Β"));

        let negated = matcher("[^k]", false, &[]);
        assert!(!negated.is_match("K"));
        assert!(negated.is_match("K"));

        let intersected = matcher("[a-z&&[^q]]", false, &[]);
        assert!(intersected.is_match("A"));
        assert!(!intersected.is_match("Q"));
        assert!(!intersected.is_match("ſ"));
    }

    #[test]
    fn folds_unicode_properties_but_preserves_marked_nested_classes() {
        let property = matcher(r"[\p{Lu}]", false, &[]);
        assert!(property.is_match("β"));

        let pattern = r"[x[\x{0}-\x{7F}]]";
        let start = pattern.find('[').expect("outer class") + 2;
        let end = pattern.len() - 1;
        let protected_spans = std::iter::once(start..end).collect::<Vec<_>>();
        let protected = matcher(pattern, true, &protected_spans);
        assert!(protected.is_match("K"));
        assert!(!protected.is_match("K"));
    }
}
