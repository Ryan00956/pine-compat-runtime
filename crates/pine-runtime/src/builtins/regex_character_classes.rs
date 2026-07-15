use std::{cmp, ops::Range};

use regex_syntax::{
    ParserBuilder,
    ast::{self, Ast, ClassBracketed, ClassSet, ClassSetBinaryOpKind, ClassSetItem, Span},
    hir::{Class, ClassUnicode, ClassUnicodeRange, HirKind},
};

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
