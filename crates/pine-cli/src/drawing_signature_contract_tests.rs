use pine_builtins::{Accepts, ReturnSpec};
use std::{fs, path::PathBuf};

const DRAWING_PREFIXES: &[&str] = &[
    "label.",
    "line.",
    "linefill.",
    "polyline.",
    "box.",
    "table.",
];

const DRAWING_ALL_VALUES: &[&str] = &[
    "label.all",
    "line.all",
    "linefill.all",
    "polyline.all",
    "box.all",
    "table.all",
];

#[test]
fn drawing_builtin_signatures_stay_in_sync_with_docs() {
    let docs_path = workspace_dir().join("docs/BUILTIN_SIGNATURES.md");
    let docs = fs::read_to_string(&docs_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", docs_path.display()));

    for signature in pine_builtins::PHASE_1_BUILTINS
        .iter()
        .filter(|signature| drawing_signature_name(signature.name))
    {
        let expected = format_signature(signature);
        assert!(
            docs.lines().any(|line| line == expected),
            "BUILTIN_SIGNATURES.md should document `{expected}`"
        );
    }

    for name in DRAWING_ALL_VALUES {
        let pine_type = pine_builtins::builtin_series_value_type(name)
            .unwrap_or_else(|| panic!("missing builtin series value for `{name}`"));
        let expected = format!("{name} -> {}", pine_type_doc(pine_type));
        assert!(
            docs.lines().any(|line| line == expected),
            "BUILTIN_SIGNATURES.md should document `{expected}`"
        );
    }
}

fn drawing_signature_name(name: &str) -> bool {
    DRAWING_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn format_signature(signature: &pine_builtins::BuiltinSignature) -> String {
    let params = signature
        .params
        .iter()
        .map(|param| {
            let optional = if param.optional { "?" } else { "" };
            format!("{}{}: {}", param.name, optional, accepts_doc(param.accepts))
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{}({params}) -> {}",
        signature.name,
        return_doc(signature.returns)
    )
}

fn accepts_doc(accepts: Accepts) -> &'static str {
    match accepts {
        Accepts::Exact(pine_type) => pine_type_doc(pine_type),
        Accepts::ConstString => "const string",
        Accepts::ConstBool => "const bool",
        Accepts::ColorCompatible => "color-compatible",
        Accepts::StringCompatible => "string-compatible",
        Accepts::StringOrIntCompatible => "string-or-int-compatible",
        Accepts::NumericCompatible => "numeric-compatible",
        Accepts::IntCompatible => "int-compatible",
        Accepts::BoolCompatible => "bool-compatible",
        Accepts::LabelCompatible => "label-compatible",
        Accepts::LineCompatible => "line-compatible",
        Accepts::LineFillCompatible => "linefill-compatible",
        Accepts::PolylineCompatible => "polyline-compatible",
        Accepts::BoxCompatible => "box-compatible",
        Accepts::TableCompatible => "table-compatible",
        Accepts::ChartPointCompatible => "chart.point-compatible",
        other => panic!("unsupported drawing signature acceptor {other:?}"),
    }
}

fn return_doc(returns: ReturnSpec) -> &'static str {
    match returns {
        ReturnSpec::Fixed(pine_type) => pine_type_doc(pine_type),
        other => panic!("unsupported drawing signature return {other:?}"),
    }
}

fn pine_type_doc(pine_type: impl std::fmt::Debug) -> &'static str {
    match format!("{pine_type:?}").as_str() {
        "PineType { qualifier: Const, kind: Void }" => "void",
        "PineType { qualifier: Series, kind: Int }" => "series int",
        "PineType { qualifier: Series, kind: Float }" => "series float",
        "PineType { qualifier: Series, kind: String }" => "series string",
        "PineType { qualifier: Series, kind: Label }" => "series label",
        "PineType { qualifier: Series, kind: Line }" => "series line",
        "PineType { qualifier: Series, kind: LineFill }" => "series linefill",
        "PineType { qualifier: Series, kind: Polyline }" => "series polyline",
        "PineType { qualifier: Series, kind: Box }" => "series box",
        "PineType { qualifier: Series, kind: Table }" => "series table",
        "PineType { qualifier: Simple, kind: ChartPointArray }" => "simple array<chart.point>",
        "PineType { qualifier: Simple, kind: LabelArray }" => "simple array<label>",
        "PineType { qualifier: Simple, kind: LineArray }" => "simple array<line>",
        "PineType { qualifier: Simple, kind: LineFillArray }" => "simple array<linefill>",
        "PineType { qualifier: Simple, kind: PolylineArray }" => "simple array<polyline>",
        "PineType { qualifier: Simple, kind: BoxArray }" => "simple array<box>",
        "PineType { qualifier: Simple, kind: TableArray }" => "simple array<table>",
        other => panic!("unsupported drawing signature type {other}"),
    }
}

fn workspace_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
