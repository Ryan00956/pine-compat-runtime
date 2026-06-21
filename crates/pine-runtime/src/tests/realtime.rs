use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn realtime_rollback_restores_math_random_state() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("realtime random")
plot(math.random(0, 1, 7))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    let forming_value = forming.plots[0].values[1].clone();

    let rolled_back = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update");
    assert_eq!(rolled_back.plots[0].values[1], forming_value);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update");
    assert_eq!(confirmed.plots[0].values[1], forming_value);
}

#[test]
fn barstate_realtime_flags_track_update_kind() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("barstate realtime")
plot(barstate.isconfirmed ? close : 0)
plot(barstate.ishistory ? close : 0)
plot(barstate.isrealtime ? close : 0)
plot(barstate.islast ? close : 0)
plot(barstate.isnew ? close : 0)
plot(barstate.islastconfirmedhistory ? close : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    let confirmed = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");
    assert_values_close(&confirmed.plots[0].values, &[1.0]);
    assert_values_close(&confirmed.plots[1].values, &[1.0]);
    assert_values_close(&confirmed.plots[2].values, &[0.0]);
    assert_values_close(&confirmed.plots[3].values, &[1.0]);
    assert_values_close(&confirmed.plots[4].values, &[1.0]);
    assert_values_close(&confirmed.plots[5].values, &[1.0]);

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[1].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[2].values, &[0.0, 2.0]);
    assert_values_close(&forming.plots[3].values, &[1.0, 2.0]);
    assert_values_close(&forming.plots[4].values, &[1.0, 2.0]);
    assert_values_close(&forming.plots[5].values, &[1.0, 0.0]);

    let forming = runtime
        .update(BarUpdate::forming(bar(4.0)))
        .expect("second forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[1].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[2].values, &[0.0, 4.0]);
    assert_values_close(&forming.plots[3].values, &[1.0, 4.0]);
    assert_values_close(&forming.plots[4].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[5].values, &[1.0, 0.0]);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(3.0)))
        .expect("confirmed update");
    assert_values_close(&confirmed.plots[0].values, &[1.0, 3.0]);
    assert_values_close(&confirmed.plots[1].values, &[1.0, 0.0]);
    assert_values_close(&confirmed.plots[2].values, &[0.0, 3.0]);
    assert_values_close(&confirmed.plots[3].values, &[1.0, 3.0]);
    assert_values_close(&confirmed.plots[4].values, &[1.0, 0.0]);
    assert_values_close(&confirmed.plots[5].values, &[1.0, 0.0]);
}

#[test]
fn realtime_forming_updates_roll_back_previous_forming_output() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("realtime")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    let first = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");
    assert_values_close(&first.plots[0].values, &[1.0]);

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 2.0]);
    assert_values_close(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let rolled_back = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update");
    assert_values_close(&rolled_back.plots[0].values, &[1.0, 3.0]);
    assert_values_close(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update");
    assert_values_close(&confirmed.plots[0].values, &[1.0, 4.0]);
    assert_eq!(runtime.profile().bars, 2);
    assert_eq!(runtime.confirmed_profile().bars, 2);
}

#[test]
fn realtime_rollback_restores_var_state() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("realtime var")
var x = 0
x := x + 1
plot(x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 2.0]);

    let rolled_back = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update");
    assert_values_close(&rolled_back.plots[0].values, &[1.0, 2.0]);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update");
    assert_values_close(&confirmed.plots[0].values, &[1.0, 2.0]);
}

#[test]
fn realtime_rollback_restores_typed_user_type_var_state() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("realtime typed UDT var")
type Point
    float x

var Point p = na
if bar_index == 0
    p := Point.new(close)
else
    p := Point.new(p.x + 1)
plot(p.x)
var Point q = if bar_index == 0
    Point.new(close + 10)
else
    Point.new(close + 20)
if bar_index > 0
    q := Point.new(q.x + 1)
plot(q.x)
var Point r = bar_index == 0 ? Point.new(close + 30) : Point.new(close + 40)
if bar_index > 0
    r := Point.new(r.x + 1)
plot(r.x)
var Point s = switch bar_index
    0 => Point.new(close + 50)
    1 => Point.new(close + 60)
    => Point.new(close + 70)
if bar_index > 0
    s := Point.new(s.x + 1)
plot(s.x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 2.0]);
    assert_values_close(&forming.plots[1].values, &[11.0, 12.0]);
    assert_values_close(&forming.plots[2].values, &[31.0, 32.0]);
    assert_values_close(&forming.plots[3].values, &[51.0, 52.0]);

    let rolled_back = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update");
    assert_values_close(&rolled_back.plots[0].values, &[1.0, 2.0]);
    assert_values_close(&rolled_back.plots[1].values, &[11.0, 12.0]);
    assert_values_close(&rolled_back.plots[2].values, &[31.0, 32.0]);
    assert_values_close(&rolled_back.plots[3].values, &[51.0, 52.0]);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update");
    assert_values_close(&confirmed.plots[0].values, &[1.0, 2.0]);
    assert_values_close(&confirmed.plots[1].values, &[11.0, 12.0]);
    assert_values_close(&confirmed.plots[2].values, &[31.0, 32.0]);
    assert_values_close(&confirmed.plots[3].values, &[51.0, 52.0]);
}
