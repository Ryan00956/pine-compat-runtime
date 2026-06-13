use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn runs_color_new_and_named_colors() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("colors")
c = color.new(color.red, 50)
opaque = color.new(color.blue)
custom = color.rgb(255, 153, 0, 50)
clamped = color.rgb(260.4, -1.4, 127.5, 125)
clamped_base = #112233
clamped_opaque = color.new(clamped_base, -10)
clamped_clear = color.new(clamped_base, 120)
gradient = color.from_gradient(close, 1, 3, color.red, color.green)
missing_gradient = color.from_gradient(na, 1, 3, color.red, color.green)
hex = #ff990080
channels = color.r(custom) + color.g(custom) + color.b(custom) + color.t(custom)
clamped_channels = color.r(clamped) + color.g(clamped) + color.b(clamped) + color.t(clamped)
clamped_transparency = color.t(clamped_opaque) + color.t(clamped_clear)
hex_channels = color.r(hex) + color.g(hex) + color.b(hex) + color.t(hex)
gradient_channels = color.r(gradient) + color.g(gradient) + color.b(gradient) + color.t(gradient)
bgcolor(custom)
plot(na(c) ? 0 : 1)
plot(opaque == color.new(color.blue, 0) ? 1 : 0)
plot(channels)
plot(clamped_channels)
plot(clamped_transparency)
plot(hex_channels)
plot(gradient_channels)
plot(na(missing_gradient) ? 1 : 0)
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

    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[458.0, 458.0]);
    assert_values_close(&result.plots[3].values, &[483.0, 483.0]);
    assert_values_close(&result.plots[4].values, &[100.0, 100.0]);
    assert_values_close(&result.plots[5].values, &[458.0, 458.0]);
    assert_values_close(&result.plots[6].values, &[255.0, 192.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
    assert_eq!(apply_transparency(0xFF0000, 50), 0xFF000080);
    assert_eq!(apply_transparency(0x112233, -10), 0x112233FF);
    assert_eq!(apply_transparency(0x112233, 120), 0x11223300);
    assert_eq!(
        result.bg_colors[0].values,
        vec![PineValue::Color(0xFF990080), PineValue::Color(0xFF990080)]
    );
}
