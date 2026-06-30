#![allow(dead_code)]

use pine_ir::HirCallArg;

use crate::*;

const MAX_MATRIX_CELLS: usize = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatrixElementKind {
    Float,
    Int,
    Bool,
    String,
    Color,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MatrixStorage {
    pub(crate) kind: MatrixElementKind,
    pub(crate) rows: usize,
    pub(crate) columns: usize,
    pub(crate) values: Vec<PineValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MatrixStoreProfile {
    pub(crate) slots: usize,
    pub(crate) capacity: usize,
    pub(crate) cells: usize,
    pub(crate) cell_capacity: usize,
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_matrix_call(
        &mut self,
        callee: &str,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        if !callee.starts_with("matrix.") {
            return None;
        }

        Some(match callee {
            "matrix.new<float>" => self.eval_matrix_new_float(args),
            "matrix.get" => self.eval_matrix_get(args),
            "matrix.set" => self.eval_matrix_set(args),
            "matrix.fill" => self.eval_matrix_fill(args),
            "matrix.copy" => self.eval_matrix_copy(args),
            "matrix.reshape" => self.eval_matrix_reshape(args),
            "matrix.add_row" => self.eval_matrix_add_row(args),
            "matrix.add_col" => self.eval_matrix_add_col(args),
            "matrix.remove_col" => self.eval_matrix_remove_col(args),
            "matrix.remove_row" => self.eval_matrix_remove_row(args),
            "matrix.rows" => self.eval_matrix_rows(args),
            "matrix.columns" => self.eval_matrix_columns(args),
            "matrix.sum" => self.eval_matrix_sum(args),
            "matrix.avg" => self.eval_matrix_avg(args),
            "matrix.row" => self.eval_matrix_row(args),
            "matrix.col" => self.eval_matrix_col(args),
            _ => return None,
        })
    }

    pub(crate) fn eval_matrix_new_float(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let rows = matrix_dimension_value("row", self.eval_expr(&args[0].value)?)?;
        let columns = matrix_dimension_value("column", self.eval_expr(&args[1].value)?)?;
        let initial_value = if let Some(initial_value) = args.get(2) {
            eval_matrix_float_value(self.eval_expr(&initial_value.value)?)
        } else {
            PineValue::Na
        };
        self.new_matrix(MatrixElementKind::Float, rows, columns, initial_value)
    }

    pub(crate) fn eval_matrix_get(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let row = self.eval_expr(&args[1].value)?;
        let column = self.eval_expr(&args[2].value)?;
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Na);
        };
        let row = matrix_index_value("row", row)?;
        let column = matrix_index_value("column", column)?;
        Ok(self
            .matrix_get_cloned(id, row, column)?
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_matrix_set(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let row = self.eval_expr(&args[1].value)?;
        let column = self.eval_expr(&args[2].value)?;
        let PineValue::Matrix(id) = id else {
            let _ = self.eval_expr(&args[3].value)?;
            return Ok(PineValue::Void);
        };
        let row = matrix_index_value("row", row)?;
        let column = matrix_index_value("column", column)?;
        let value = eval_matrix_float_value(self.eval_expr(&args[3].value)?);
        self.matrix_set_value(id, row, column, value)?;
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_matrix_fill(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let value = eval_matrix_float_value(self.eval_expr(&args[1].value)?);
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Void);
        };
        self.matrix_fill_value(id, value);
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_matrix_copy(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::Matrix(id) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        Ok(self.copy_matrix(id))
    }

    pub(crate) fn eval_matrix_reshape(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let rows = matrix_dimension_value("row", self.eval_expr(&args[1].value)?)?;
        let columns = matrix_dimension_value("column", self.eval_expr(&args[2].value)?)?;
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Void);
        };
        self.matrix_reshape(id, rows, columns)?;
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_matrix_add_row(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let row = matrix_insert_index_value("row", self.eval_expr(&args[1].value)?)?;
        let array_id = self.eval_expr(&args[2].value)?;
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Void);
        };
        let PineValue::Array(array_id) = array_id else {
            return Ok(PineValue::Void);
        };
        let Some(values) = self.array_values_clone(array_id)? else {
            return Ok(PineValue::Void);
        };
        self.matrix_add_row(id, row, values)?;
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_matrix_add_col(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let column = matrix_insert_index_value("column", self.eval_expr(&args[1].value)?)?;
        let array_id = self.eval_expr(&args[2].value)?;
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Void);
        };
        let PineValue::Array(array_id) = array_id else {
            return Ok(PineValue::Void);
        };
        let Some(values) = self.array_values_clone(array_id)? else {
            return Ok(PineValue::Void);
        };
        self.matrix_add_col(id, column, values)?;
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_matrix_remove_row(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let row = matrix_index_value("row", self.eval_expr(&args[1].value)?)?;
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Void);
        };
        self.matrix_remove_row(id, row)?;
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_matrix_remove_col(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let column = matrix_index_value("column", self.eval_expr(&args[1].value)?)?;
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Void);
        };
        self.matrix_remove_col(id, column)?;
        Ok(PineValue::Void)
    }

    pub(crate) fn eval_matrix_rows(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::Matrix(id) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .matrix_shape(id)
            .map(|(rows, _)| PineValue::Int(rows as i64))
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_matrix_columns(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::Matrix(id) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        Ok(self
            .matrix_shape(id)
            .map(|(_, columns)| PineValue::Int(columns as i64))
            .unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_matrix_sum(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::Matrix(id) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        Ok(self.matrix_sum(id).unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_matrix_avg(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let PineValue::Matrix(id) = self.eval_expr(&args[0].value)? else {
            return Ok(PineValue::Na);
        };
        Ok(self.matrix_avg(id).unwrap_or(PineValue::Na))
    }

    pub(crate) fn eval_matrix_row(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let row = self.eval_expr(&args[1].value)?;
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Na);
        };
        let row = matrix_index_value("row", row)?;
        let Some(values) = self.matrix_row_values(id, row)? else {
            return Ok(PineValue::Na);
        };
        Ok(self.new_array_from_values(ArrayElementKind::Float, values))
    }

    pub(crate) fn eval_matrix_col(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_expr(&args[0].value)?;
        let column = self.eval_expr(&args[1].value)?;
        let PineValue::Matrix(id) = id else {
            return Ok(PineValue::Na);
        };
        let column = matrix_index_value("column", column)?;
        let Some(values) = self.matrix_col_values(id, column)? else {
            return Ok(PineValue::Na);
        };
        Ok(self.new_array_from_values(ArrayElementKind::Float, values))
    }

    pub(crate) fn new_matrix(
        &mut self,
        kind: MatrixElementKind,
        rows: i64,
        columns: i64,
        initial_value: PineValue,
    ) -> Result<PineValue, RuntimeError> {
        let rows = matrix_dimension("row", rows)?;
        let columns = matrix_dimension("column", columns)?;
        let cells = rows.checked_mul(columns).ok_or_else(|| RuntimeError {
            message: format!("matrix cell count cannot exceed {MAX_MATRIX_CELLS}"),
        })?;
        if cells > MAX_MATRIX_CELLS {
            return Err(RuntimeError {
                message: format!("matrix cell count cannot exceed {MAX_MATRIX_CELLS}"),
            });
        }

        let id = self.next_matrix_id;
        self.next_matrix_id += 1;
        self.matrix_store.insert(
            id,
            MatrixStorage {
                kind,
                rows,
                columns,
                values: vec![initial_value; cells],
            },
        );
        Ok(PineValue::Matrix(id))
    }

    pub(crate) fn matrix_shape(&self, id: u32) -> Option<(usize, usize)> {
        self.matrix_store
            .get(&id)
            .map(|matrix| (matrix.rows, matrix.columns))
    }

    pub(crate) fn matrix_get_cloned(
        &self,
        id: u32,
        row: i64,
        column: i64,
    ) -> Result<Option<PineValue>, RuntimeError> {
        let Some((matrix, offset)) = self.matrix_cell_offset(id, row, column)? else {
            return Ok(None);
        };
        Ok(matrix.values.get(offset).cloned())
    }

    pub(crate) fn matrix_set_value(
        &mut self,
        id: u32,
        row: i64,
        column: i64,
        value: PineValue,
    ) -> Result<(), RuntimeError> {
        let Some(offset) = self
            .matrix_cell_offset(id, row, column)?
            .map(|(_, offset)| offset)
        else {
            return Ok(());
        };
        if let Some(slot) = self
            .matrix_store
            .get_mut(&id)
            .and_then(|matrix| matrix.values.get_mut(offset))
        {
            *slot = value;
        }
        Ok(())
    }

    pub(crate) fn matrix_fill_value(&mut self, id: u32, value: PineValue) {
        if let Some(matrix) = self.matrix_store.get_mut(&id) {
            matrix.values.fill(value);
        }
    }

    pub(crate) fn copy_matrix(&mut self, source_id: u32) -> PineValue {
        let Some(source) = self.matrix_store.get(&source_id).cloned() else {
            return PineValue::Na;
        };
        let id = self.next_matrix_id;
        self.next_matrix_id += 1;
        self.matrix_store.insert(id, source);
        PineValue::Matrix(id)
    }

    pub(crate) fn matrix_reshape(
        &mut self,
        id: u32,
        rows: i64,
        columns: i64,
    ) -> Result<(), RuntimeError> {
        let rows = matrix_dimension("row", rows)?;
        let columns = matrix_dimension("column", columns)?;
        let cells = rows.checked_mul(columns).ok_or_else(|| RuntimeError {
            message: "matrix reshape dimensions must preserve element count".to_owned(),
        })?;
        let Some(matrix) = self.matrix_store.get_mut(&id) else {
            return Ok(());
        };
        if cells != matrix.values.len() {
            return Err(RuntimeError {
                message: "matrix reshape dimensions must preserve element count".to_owned(),
            });
        }
        matrix.rows = rows;
        matrix.columns = columns;
        Ok(())
    }

    pub(crate) fn matrix_add_row(
        &mut self,
        id: u32,
        row: i64,
        values: Vec<PineValue>,
    ) -> Result<(), RuntimeError> {
        let Some(matrix) = self.matrix_store.get_mut(&id) else {
            return Ok(());
        };
        let row = matrix_insert_index("row", row, matrix.rows)?;
        if values.len() != matrix.columns {
            return Err(RuntimeError {
                message: format!(
                    "matrix add_row array size {} must match column count {}",
                    values.len(),
                    matrix.columns
                ),
            });
        }
        let new_cells = (matrix.rows + 1)
            .checked_mul(matrix.columns)
            .ok_or_else(|| RuntimeError {
                message: format!("matrix cell count cannot exceed {MAX_MATRIX_CELLS}"),
            })?;
        if new_cells > MAX_MATRIX_CELLS {
            return Err(RuntimeError {
                message: format!("matrix cell count cannot exceed {MAX_MATRIX_CELLS}"),
            });
        }
        let offset = row * matrix.columns;
        matrix.values.splice(
            offset..offset,
            values.into_iter().map(eval_matrix_float_value),
        );
        matrix.rows += 1;
        Ok(())
    }

    pub(crate) fn matrix_add_col(
        &mut self,
        id: u32,
        column: i64,
        values: Vec<PineValue>,
    ) -> Result<(), RuntimeError> {
        let Some(matrix) = self.matrix_store.get_mut(&id) else {
            return Ok(());
        };
        let column = matrix_insert_index("column", column, matrix.columns)?;
        if values.len() != matrix.rows {
            return Err(RuntimeError {
                message: format!(
                    "matrix add_col array size {} must match row count {}",
                    values.len(),
                    matrix.rows
                ),
            });
        }
        let new_columns = matrix.columns + 1;
        let new_cells = matrix
            .rows
            .checked_mul(new_columns)
            .ok_or_else(|| RuntimeError {
                message: format!("matrix cell count cannot exceed {MAX_MATRIX_CELLS}"),
            })?;
        if new_cells > MAX_MATRIX_CELLS {
            return Err(RuntimeError {
                message: format!("matrix cell count cannot exceed {MAX_MATRIX_CELLS}"),
            });
        }

        let mut inserted_values = values.into_iter().map(eval_matrix_float_value);
        let mut next_values = Vec::with_capacity(new_cells);
        for row in 0..matrix.rows {
            let start = row * matrix.columns;
            let insert_offset = start + column;
            next_values.extend_from_slice(&matrix.values[start..insert_offset]);
            next_values.push(inserted_values.next().unwrap_or(PineValue::Na));
            next_values.extend_from_slice(&matrix.values[insert_offset..start + matrix.columns]);
        }
        matrix.columns = new_columns;
        matrix.values = next_values;
        Ok(())
    }

    pub(crate) fn matrix_remove_row(&mut self, id: u32, row: i64) -> Result<(), RuntimeError> {
        let Some(matrix) = self.matrix_store.get_mut(&id) else {
            return Ok(());
        };
        let row = matrix_index("row", row, matrix.rows)?;
        let start = row * matrix.columns;
        let end = start + matrix.columns;
        matrix.values.drain(start..end);
        matrix.rows -= 1;
        Ok(())
    }

    pub(crate) fn matrix_remove_col(&mut self, id: u32, column: i64) -> Result<(), RuntimeError> {
        let Some(matrix) = self.matrix_store.get_mut(&id) else {
            return Ok(());
        };
        let column = matrix_index("column", column, matrix.columns)?;
        let new_columns = matrix.columns - 1;
        let mut next_values = Vec::with_capacity(matrix.rows * new_columns);
        for row in 0..matrix.rows {
            let start = row * matrix.columns;
            let remove_offset = start + column;
            next_values.extend_from_slice(&matrix.values[start..remove_offset]);
            next_values
                .extend_from_slice(&matrix.values[remove_offset + 1..start + matrix.columns]);
        }
        matrix.columns = new_columns;
        matrix.values = next_values;
        Ok(())
    }

    pub(crate) fn matrix_row_values(
        &self,
        id: u32,
        row: i64,
    ) -> Result<Option<Vec<PineValue>>, RuntimeError> {
        let Some(matrix) = self.matrix_store.get(&id) else {
            return Ok(None);
        };
        let row = matrix_index("row", row, matrix.rows)?;
        let start = row * matrix.columns;
        let end = start + matrix.columns;
        Ok(Some(matrix.values[start..end].to_vec()))
    }

    pub(crate) fn matrix_col_values(
        &self,
        id: u32,
        column: i64,
    ) -> Result<Option<Vec<PineValue>>, RuntimeError> {
        let Some(matrix) = self.matrix_store.get(&id) else {
            return Ok(None);
        };
        let column = matrix_index("column", column, matrix.columns)?;
        Ok(Some(
            (0..matrix.rows)
                .map(|row| matrix.values[row * matrix.columns + column].clone())
                .collect(),
        ))
    }

    pub(crate) fn matrix_sum(&self, id: u32) -> Option<PineValue> {
        let matrix = self.matrix_store.get(&id)?;
        let mut total = 0.0;
        let mut has_value = false;
        for value in &matrix.values {
            if let Some(number) = value.as_f64() {
                total += number;
                has_value = true;
            }
        }
        has_value.then(|| finite_float_or_na(total))
    }

    pub(crate) fn matrix_avg(&self, id: u32) -> Option<PineValue> {
        let matrix = self.matrix_store.get(&id)?;
        let mut total = 0.0;
        let mut count = 0_usize;
        for value in &matrix.values {
            if let Some(number) = value.as_f64() {
                total += number;
                count += 1;
            }
        }
        (count > 0).then(|| finite_float_or_na(total / count as f64))
    }

    pub(crate) fn matrix_store_profile(&self) -> MatrixStoreProfile {
        MatrixStoreProfile {
            slots: self.matrix_store.len(),
            capacity: self.matrix_store.capacity(),
            cells: self
                .matrix_store
                .values()
                .map(|matrix| matrix.values.len())
                .sum(),
            cell_capacity: self
                .matrix_store
                .values()
                .map(|matrix| matrix.values.capacity())
                .sum(),
        }
    }

    fn matrix_cell_offset(
        &self,
        id: u32,
        row: i64,
        column: i64,
    ) -> Result<Option<(&MatrixStorage, usize)>, RuntimeError> {
        let Some(matrix) = self.matrix_store.get(&id) else {
            return Ok(None);
        };
        let row = matrix_index("row", row, matrix.rows)?;
        let column = matrix_index("column", column, matrix.columns)?;
        Ok(Some((matrix, row * matrix.columns + column)))
    }
}

fn eval_matrix_float_value(value: PineValue) -> PineValue {
    match value {
        PineValue::Int(value) => PineValue::Float(value as f64),
        PineValue::Float(_) | PineValue::Na => value,
        _ => PineValue::Na,
    }
}

fn matrix_dimension(name: &str, value: i64) -> Result<usize, RuntimeError> {
    if value < 0 {
        return Err(RuntimeError {
            message: format!("matrix {name} count cannot be negative"),
        });
    }
    Ok(value as usize)
}

fn matrix_dimension_value(name: &str, value: PineValue) -> Result<i64, RuntimeError> {
    value.as_i64().ok_or_else(|| RuntimeError {
        message: format!("matrix {name} count cannot be na"),
    })
}

fn matrix_index_value(name: &str, value: PineValue) -> Result<i64, RuntimeError> {
    value.as_i64().ok_or_else(|| RuntimeError {
        message: format!("matrix {name} index cannot be na"),
    })
}

fn matrix_insert_index_value(name: &str, value: PineValue) -> Result<i64, RuntimeError> {
    value.as_i64().ok_or_else(|| RuntimeError {
        message: format!("matrix {name} index cannot be na"),
    })
}

fn matrix_index(name: &str, value: i64, len: usize) -> Result<usize, RuntimeError> {
    if value < 0 || value as usize >= len {
        return Err(RuntimeError {
            message: format!("matrix {name} index {value} is out of bounds for size {len}"),
        });
    }
    Ok(value as usize)
}

fn matrix_insert_index(name: &str, value: i64, len: usize) -> Result<usize, RuntimeError> {
    if value < 0 || value as usize > len {
        return Err(RuntimeError {
            message: format!(
                "matrix {name} index {value} is out of bounds for size {}",
                len + 1
            ),
        });
    }
    Ok(value as usize)
}
