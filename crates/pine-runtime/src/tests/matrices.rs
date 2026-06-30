use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use crate::builtins::matrices::MatrixElementKind;

use super::*;

fn runtime_program() -> pine_ir::HirProgram {
    let source = SourceFile::new("test.pine", "indicator(\"matrix runtime scaffold\")\n");
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.expect("HIR")
}

#[test]
fn allocates_rectangular_matrix_storage() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);

    let value = runtime
        .new_matrix(MatrixElementKind::Float, 2, 3, PineValue::Float(1.5))
        .expect("matrix allocation");

    let PineValue::Matrix(id) = value else {
        panic!("expected matrix id, got {value:?}");
    };
    assert_eq!(runtime.matrix_shape(id), Some((2, 3)));
    assert_eq!(
        runtime
            .matrix_get_cloned(id, 1, 2)
            .expect("matrix get should succeed"),
        Some(PineValue::Float(1.5))
    );
    let profile = runtime.matrix_store_profile();
    assert_eq!(profile.slots, 1);
    assert_eq!(profile.cells, 6);
    assert!(profile.capacity >= profile.slots);
    assert!(profile.cell_capacity >= profile.cells);
}

#[test]
fn writes_matrix_cells_without_changing_shape() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Int, 2, 2, PineValue::Int(0))
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };

    runtime
        .matrix_set_value(id, 1, 0, PineValue::Int(42))
        .expect("matrix set should succeed");

    assert_eq!(runtime.matrix_shape(id), Some((2, 2)));
    assert_eq!(
        runtime
            .matrix_get_cloned(id, 1, 0)
            .expect("matrix get should succeed"),
        Some(PineValue::Int(42))
    );
    assert_eq!(
        runtime
            .matrix_get_cloned(id, 0, 0)
            .expect("matrix get should succeed"),
        Some(PineValue::Int(0))
    );
}

#[test]
fn fills_matrix_cells_without_changing_shape() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 2, 2, PineValue::Float(1.0))
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };

    runtime.matrix_fill_value(id, PineValue::Float(7.5));

    assert_eq!(runtime.matrix_shape(id), Some((2, 2)));
    for row in 0..2 {
        for column in 0..2 {
            assert_eq!(
                runtime
                    .matrix_get_cloned(id, row, column)
                    .expect("matrix get should succeed"),
                Some(PineValue::Float(7.5))
            );
        }
    }
}

#[test]
fn summarizes_numeric_matrix_cells_and_ignores_na() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 2, 2, PineValue::Na)
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };

    runtime
        .matrix_set_value(id, 0, 0, PineValue::Float(1.5))
        .expect("matrix set should succeed");
    runtime
        .matrix_set_value(id, 1, 1, PineValue::Int(2))
        .expect("matrix set should succeed");

    assert_eq!(runtime.matrix_sum(id), Some(PineValue::Float(3.5)));
    assert_eq!(runtime.matrix_avg(id), Some(PineValue::Float(1.75)));

    let PineValue::Matrix(empty_id) = runtime
        .new_matrix(MatrixElementKind::Float, 0, 2, PineValue::Na)
        .expect("empty matrix allocation")
    else {
        panic!("expected matrix id");
    };
    assert_eq!(runtime.matrix_sum(empty_id), None);
    assert_eq!(runtime.matrix_avg(empty_id), None);
}

#[test]
fn copies_matrix_storage_independently() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(source_id) = runtime
        .new_matrix(
            MatrixElementKind::String,
            1,
            2,
            PineValue::String("a".to_owned()),
        )
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };

    let PineValue::Matrix(copy_id) = runtime.copy_matrix(source_id) else {
        panic!("expected copied matrix id");
    };
    runtime
        .matrix_set_value(copy_id, 0, 1, PineValue::String("b".to_owned()))
        .expect("copy mutation should succeed");

    assert_ne!(source_id, copy_id);
    assert_eq!(
        runtime
            .matrix_get_cloned(source_id, 0, 1)
            .expect("source get should succeed"),
        Some(PineValue::String("a".to_owned()))
    );
    assert_eq!(
        runtime
            .matrix_get_cloned(copy_id, 0, 1)
            .expect("copy get should succeed"),
        Some(PineValue::String("b".to_owned()))
    );
    assert_eq!(runtime.matrix_store_profile().slots, 2);
}

#[test]
fn reshapes_matrix_without_reordering_cells() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 2, 3, PineValue::Float(0.0))
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };

    for row in 0..2 {
        for column in 0..3 {
            runtime
                .matrix_set_value(id, row, column, PineValue::Float((row * 3 + column) as f64))
                .expect("matrix set should succeed");
        }
    }

    runtime
        .matrix_reshape(id, 3, 2)
        .expect("reshape should preserve element count");

    assert_eq!(runtime.matrix_shape(id), Some((3, 2)));
    assert_eq!(
        runtime
            .matrix_get_cloned(id, 2, 1)
            .expect("matrix get should succeed"),
        Some(PineValue::Float(5.0))
    );
}

#[test]
fn inserts_matrix_rows_from_copied_array_values() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 2, 2, PineValue::Float(1.0))
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };
    runtime
        .matrix_set_value(id, 1, 0, PineValue::Float(3.0))
        .expect("matrix set should succeed");
    runtime
        .matrix_set_value(id, 1, 1, PineValue::Float(4.0))
        .expect("matrix set should succeed");

    runtime
        .matrix_add_row(id, 1, vec![PineValue::Float(9.0), PineValue::Int(10)])
        .expect("matrix add_row should succeed");

    assert_eq!(runtime.matrix_shape(id), Some((3, 2)));
    assert_eq!(
        runtime
            .matrix_row_values(id, 1)
            .expect("matrix row should succeed"),
        Some(vec![PineValue::Float(9.0), PineValue::Float(10.0)])
    );
    assert_eq!(
        runtime
            .matrix_row_values(id, 2)
            .expect("matrix row should succeed"),
        Some(vec![PineValue::Float(3.0), PineValue::Float(4.0)])
    );

    let mismatch = runtime
        .matrix_add_row(id, 0, vec![PineValue::Float(1.0)])
        .expect_err("short row should fail");
    assert_eq!(
        mismatch.message,
        "matrix add_row array size 1 must match column count 2"
    );
}

#[test]
fn inserts_matrix_columns_from_copied_array_values() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 2, 2, PineValue::Float(1.0))
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };
    runtime
        .matrix_set_value(id, 0, 1, PineValue::Float(2.0))
        .expect("matrix set should succeed");
    runtime
        .matrix_set_value(id, 1, 0, PineValue::Float(3.0))
        .expect("matrix set should succeed");
    runtime
        .matrix_set_value(id, 1, 1, PineValue::Float(4.0))
        .expect("matrix set should succeed");

    runtime
        .matrix_add_col(id, 1, vec![PineValue::Float(9.0), PineValue::Int(10)])
        .expect("matrix add_col should succeed");

    assert_eq!(runtime.matrix_shape(id), Some((2, 3)));
    assert_eq!(
        runtime
            .matrix_col_values(id, 1)
            .expect("matrix col should succeed"),
        Some(vec![PineValue::Float(9.0), PineValue::Float(10.0)])
    );
    assert_eq!(
        runtime
            .matrix_row_values(id, 0)
            .expect("matrix row should succeed"),
        Some(vec![
            PineValue::Float(1.0),
            PineValue::Float(9.0),
            PineValue::Float(2.0),
        ])
    );
    assert_eq!(
        runtime
            .matrix_row_values(id, 1)
            .expect("matrix row should succeed"),
        Some(vec![
            PineValue::Float(3.0),
            PineValue::Float(10.0),
            PineValue::Float(4.0),
        ])
    );

    let mismatch = runtime
        .matrix_add_col(id, 0, vec![PineValue::Float(1.0)])
        .expect_err("short column should fail");
    assert_eq!(
        mismatch.message,
        "matrix add_col array size 1 must match row count 2"
    );
}

#[test]
fn removes_matrix_rows_and_preserves_row_major_values() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 3, 2, PineValue::Na)
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };
    for row in 0..3 {
        for column in 0..2 {
            runtime
                .matrix_set_value(
                    id,
                    row,
                    column,
                    PineValue::Float((row * 10 + column) as f64),
                )
                .expect("matrix set should succeed");
        }
    }

    runtime
        .matrix_remove_row(id, 1)
        .expect("matrix remove_row should succeed");

    assert_eq!(runtime.matrix_shape(id), Some((2, 2)));
    assert_eq!(
        runtime
            .matrix_row_values(id, 0)
            .expect("matrix row should succeed"),
        Some(vec![PineValue::Float(0.0), PineValue::Float(1.0)])
    );
    assert_eq!(
        runtime
            .matrix_row_values(id, 1)
            .expect("matrix row should succeed"),
        Some(vec![PineValue::Float(20.0), PineValue::Float(21.0)])
    );

    let out_of_bounds = runtime
        .matrix_remove_row(id, 2)
        .expect_err("row beyond shrunken matrix should fail");
    assert_eq!(
        out_of_bounds.message,
        "matrix row index 2 is out of bounds for size 2"
    );
}

#[test]
fn removes_matrix_columns_and_preserves_row_major_values() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 2, 3, PineValue::Na)
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };
    for row in 0..2 {
        for column in 0..3 {
            runtime
                .matrix_set_value(
                    id,
                    row,
                    column,
                    PineValue::Float((row * 10 + column) as f64),
                )
                .expect("matrix set should succeed");
        }
    }

    runtime
        .matrix_remove_col(id, 1)
        .expect("matrix remove_col should succeed");

    assert_eq!(runtime.matrix_shape(id), Some((2, 2)));
    assert_eq!(
        runtime
            .matrix_row_values(id, 0)
            .expect("matrix row should succeed"),
        Some(vec![PineValue::Float(0.0), PineValue::Float(2.0)])
    );
    assert_eq!(
        runtime
            .matrix_row_values(id, 1)
            .expect("matrix row should succeed"),
        Some(vec![PineValue::Float(10.0), PineValue::Float(12.0)])
    );

    let out_of_bounds = runtime
        .matrix_remove_col(id, 2)
        .expect_err("column beyond shrunken matrix should fail");
    assert_eq!(
        out_of_bounds.message,
        "matrix column index 2 is out of bounds for size 2"
    );
}

#[test]
fn copies_matrix_row_values_into_float_array() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 2, 2, PineValue::Na)
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };
    runtime
        .matrix_set_value(id, 1, 0, PineValue::Float(3.0))
        .expect("matrix set should succeed");
    runtime
        .matrix_set_value(id, 1, 1, PineValue::Float(4.0))
        .expect("matrix set should succeed");

    assert_eq!(
        runtime
            .matrix_row_values(id, 1)
            .expect("matrix row should succeed"),
        Some(vec![PineValue::Float(3.0), PineValue::Float(4.0)])
    );
}

#[test]
fn copies_matrix_column_values_into_float_array() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);
    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Float, 2, 3, PineValue::Na)
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };
    runtime
        .matrix_set_value(id, 0, 1, PineValue::Float(2.0))
        .expect("matrix set should succeed");
    runtime
        .matrix_set_value(id, 1, 1, PineValue::Float(5.0))
        .expect("matrix set should succeed");

    assert_eq!(
        runtime
            .matrix_col_values(id, 1)
            .expect("matrix column should succeed"),
        Some(vec![PineValue::Float(2.0), PineValue::Float(5.0)])
    );
}

#[test]
fn rejects_invalid_matrix_dimensions_and_indexes() {
    let program = runtime_program();
    let mut runtime = HistoricalRuntime::new(&program);

    let negative = runtime
        .new_matrix(MatrixElementKind::Bool, -1, 2, PineValue::Bool(false))
        .expect_err("negative rows should fail");
    assert_eq!(negative.message, "matrix row count cannot be negative");

    let too_large = runtime
        .new_matrix(
            MatrixElementKind::Color,
            100_001,
            1,
            PineValue::Color(0xff00ffff),
        )
        .expect_err("too many cells should fail");
    assert_eq!(too_large.message, "matrix cell count cannot exceed 100000");

    let PineValue::Matrix(id) = runtime
        .new_matrix(MatrixElementKind::Bool, 1, 1, PineValue::Bool(false))
        .expect("matrix allocation")
    else {
        panic!("expected matrix id");
    };
    let bad_row = runtime
        .matrix_get_cloned(id, 1, 0)
        .expect_err("row out of bounds should fail");
    assert_eq!(
        bad_row.message,
        "matrix row index 1 is out of bounds for size 1"
    );
    let bad_column = runtime
        .matrix_set_value(id, 0, -1, PineValue::Bool(true))
        .expect_err("negative column should fail");
    assert_eq!(
        bad_column.message,
        "matrix column index -1 is out of bounds for size 1"
    );
}
