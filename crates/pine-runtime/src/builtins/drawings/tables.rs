use pine_ir::HirCallArg;

use crate::builtins::args::call_arg_expr;
use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(super) fn eval_table_new(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let position = self.eval_required_table_arg(args, 0, "position")?;
        let columns = self.eval_required_table_int_arg(args, 1, "columns")?;
        let rows = self.eval_required_table_int_arg(args, 2, "rows")?;
        if columns <= 0 || rows <= 0 {
            return Err(RuntimeError {
                message: "table dimensions must be positive".to_owned(),
            });
        }
        let Some(cells) = columns.checked_mul(rows) else {
            return Err(RuntimeError {
                message: "table cell count overflow".to_owned(),
            });
        };
        if cells > MAX_TABLE_CELLS {
            return Err(RuntimeError {
                message: format!("table cell count cannot exceed {MAX_TABLE_CELLS}"),
            });
        }
        if self.tables.len() >= MAX_TABLES {
            return Err(RuntimeError {
                message: format!("table count cannot exceed {MAX_TABLES}"),
            });
        }
        let id = self.next_table_id;
        self.next_table_id = self
            .next_table_id
            .checked_add(1)
            .ok_or_else(|| RuntimeError {
                message: "table id limit exceeded".to_owned(),
            })?;
        self.tables.push(TableOutput {
            id,
            position,
            columns,
            rows,
            snapshots: vec![TableSnapshot {
                bar_index: self.bars,
                cells: Vec::new(),
            }],
        });
        Ok(PineValue::Table(id))
    }

    pub(super) fn eval_table_cell(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_table_id_arg(args)?;
        let column = self.eval_required_table_int_arg(args, 1, "column")?;
        let row = self.eval_required_table_int_arg(args, 2, "row")?;
        let text = self.eval_required_table_arg(args, 3, "text")?;
        let bg_color = self.eval_table_option_value(args, 4, "bgcolor", PineValue::Na)?;
        let text_color = self.eval_table_option_value(args, 5, "text_color", PineValue::Na)?;
        let Some(id) = id else {
            return Ok(PineValue::Void);
        };
        let next_cell = TableCellSnapshot {
            column,
            row,
            text,
            bg_color,
            text_color,
        };
        self.mutate_table_cell(id, column, row, true, |cell| *cell = next_cell)?;
        Ok(PineValue::Void)
    }

    pub(super) fn eval_table_cell_set_text(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_table_id_arg(args)?;
        let column = self.eval_required_table_int_arg(args, 1, "column")?;
        let row = self.eval_required_table_int_arg(args, 2, "row")?;
        let text = self.eval_required_table_arg(args, 3, "text")?;
        let Some(id) = id else {
            return Ok(PineValue::Void);
        };
        self.mutate_table_cell(id, column, row, false, |cell| {
            cell.text = text;
        })?;
        Ok(PineValue::Void)
    }

    fn eval_table_id_arg(&mut self, args: &[HirCallArg]) -> Result<Option<u32>, RuntimeError> {
        let Some(id_arg) = call_arg_expr(args, 0, "id") else {
            return Err(RuntimeError {
                message: "table mutation missing id argument".to_owned(),
            });
        };
        match self.eval_expr(id_arg)? {
            PineValue::Table(id) => Ok(Some(id)),
            PineValue::Na => Ok(None),
            value => Err(RuntimeError {
                message: format!("table mutation expected table id, got {value:?}"),
            }),
        }
    }

    fn eval_required_table_arg(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
    ) -> Result<PineValue, RuntimeError> {
        let Some(arg) = call_arg_expr(args, index, name) else {
            return Err(RuntimeError {
                message: format!("table call missing {name} argument"),
            });
        };
        self.eval_expr(arg)
    }

    fn eval_required_table_int_arg(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
    ) -> Result<i64, RuntimeError> {
        match self.eval_required_table_arg(args, index, name)? {
            PineValue::Int(value) => Ok(value),
            PineValue::Na => Err(RuntimeError {
                message: format!("table call expected integer {name}, got na"),
            }),
            value => Err(RuntimeError {
                message: format!("table call expected integer {name}, got {value:?}"),
            }),
        }
    }

    fn eval_table_option_value(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
        default: PineValue,
    ) -> Result<PineValue, RuntimeError> {
        match call_arg_expr(args, index, name) {
            Some(expr) => self.eval_expr(expr),
            None => Ok(default),
        }
    }

    fn mutate_table_cell<F>(
        &mut self,
        id: u32,
        column: i64,
        row: i64,
        create_missing: bool,
        mutate: F,
    ) -> Result<(), RuntimeError>
    where
        F: FnOnce(&mut TableCellSnapshot),
    {
        let Some(table) = self.tables.iter_mut().find(|table| table.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid table id `{id}`"),
            });
        };
        if column < 0 || column >= table.columns || row < 0 || row >= table.rows {
            return Err(RuntimeError {
                message: format!("table cell coordinate out of bounds `{column},{row}`"),
            });
        }
        let Some(latest) = table.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("table `{id}` has no snapshots"),
            });
        };
        let mut next = latest.clone();
        match next
            .cells
            .iter_mut()
            .find(|cell| cell.column == column && cell.row == row)
        {
            Some(cell) => mutate(cell),
            None if create_missing => {
                let mut cell = TableCellSnapshot {
                    column,
                    row,
                    text: PineValue::String(String::new()),
                    bg_color: PineValue::Na,
                    text_color: PineValue::Na,
                };
                mutate(&mut cell);
                next.cells.push(cell);
            }
            None => {
                return Err(RuntimeError {
                    message: format!("table cell `{column},{row}` has not been populated"),
                });
            }
        }
        next.cells.sort_by_key(|cell| (cell.row, cell.column));
        if next != latest {
            next.bar_index = self.bars;
            table.snapshots.push(next);
        }
        Ok(())
    }
}
