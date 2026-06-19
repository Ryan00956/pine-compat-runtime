use pine_ir::{HirExpr, HirStmt, HirStmtKind, SymbolId};

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StmtControl {
    None,
    Break,
    Continue,
}

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_stmt(&mut self, statement: &HirStmt) -> Result<StmtControl, RuntimeError> {
        match &statement.kind {
            HirStmtKind::Expr(expr) => {
                self.eval_expr(expr)?;
            }
            HirStmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let branch = match self.eval_expr(condition)? {
                    PineValue::Bool(true) => then_branch,
                    PineValue::Bool(false) | PineValue::Na => else_branch,
                    _ => return Ok(StmtControl::None),
                };
                for statement in branch {
                    match self.eval_stmt(statement)? {
                        StmtControl::None => {}
                        control => return Ok(control),
                    }
                }
            }
            HirStmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => {
                self.eval_for_loop(*counter, from, to, step.as_ref(), body, None)?;
            }
            HirStmtKind::While { condition, body } => {
                self.eval_while_loop(condition, body)?;
            }
            HirStmtKind::Break => return Ok(StmtControl::Break),
            HirStmtKind::Continue => return Ok(StmtControl::Continue),
            HirStmtKind::Decl { symbol, value } => {
                let value = self.eval_decl(*symbol, value)?;
                self.set_symbol_value(*symbol, value);
            }
            HirStmtKind::Reassign { symbol, value } => {
                let value = self.eval_expr(value)?;
                self.assign_persistent_symbol(*symbol, value.clone());
                self.set_symbol_value(*symbol, value);
            }
            HirStmtKind::FieldReassign {
                symbol,
                field_index,
                value,
            } => {
                let value = self.eval_expr(value)?;
                let updated = match self.current_symbols.get(symbol).cloned() {
                    Some(PineValue::UserType(mut fields)) => {
                        if *field_index < fields.len() {
                            fields[*field_index] = value;
                        }
                        PineValue::UserType(fields)
                    }
                    Some(PineValue::ChartPoint(mut point)) => {
                        point.set_field(
                            *field_index,
                            crate::builtins::chart_points::normalize_chart_point_field(
                                *field_index,
                                value,
                            ),
                        );
                        PineValue::ChartPoint(point)
                    }
                    Some(PineValue::Na) | None => return Ok(StmtControl::None),
                    Some(_) => {
                        return Err(RuntimeError {
                            message: "field mutation receiver is not an object value".to_owned(),
                        });
                    }
                };
                self.assign_persistent_symbol(*symbol, updated.clone());
                self.set_symbol_value(*symbol, updated);
            }
            HirStmtKind::TupleDecl { symbols, value } => {
                let value = self.eval_expr(value)?;
                let PineValue::Tuple(values) = value else {
                    return Ok(StmtControl::None);
                };
                for (symbol, value) in symbols.iter().zip(values) {
                    self.set_symbol_value(*symbol, value);
                }
            }
        }

        Ok(StmtControl::None)
    }

    pub(crate) fn eval_for_loop(
        &mut self,
        counter: SymbolId,
        from: &HirExpr,
        to: &HirExpr,
        step: Option<&HirExpr>,
        body: &[HirStmt],
        result: Option<&HirExpr>,
    ) -> Result<PineValue, RuntimeError> {
        let Some(from) = self.eval_expr(from)?.as_i64() else {
            return Ok(PineValue::Na);
        };
        let Some(to) = self.eval_expr(to)?.as_i64() else {
            return Ok(PineValue::Na);
        };
        let step_size = if let Some(step) = step {
            let Some(step) = self.eval_expr(step)?.as_i64() else {
                return Ok(PineValue::Na);
            };
            if step == 0 {
                return Err(RuntimeError {
                    message: "for loop step cannot be zero".to_owned(),
                });
            }
            step.checked_abs().ok_or_else(|| RuntimeError {
                message: "for loop step is out of range".to_owned(),
            })?
        } else {
            1
        };
        let step = if from <= to { step_size } else { -step_size };
        let mut value = from;
        let mut loop_result = PineValue::Na;
        loop {
            if (step > 0 && value > to) || (step < 0 && value < to) {
                break;
            }
            self.set_symbol_value(counter, PineValue::Int(value));
            let mut control = StmtControl::None;
            for statement in body {
                match self.eval_stmt(statement)? {
                    StmtControl::None => {}
                    next_control => {
                        control = next_control;
                        break;
                    }
                }
            }
            match control {
                StmtControl::None => {
                    if let Some(result) = result {
                        loop_result = self.eval_expr(result)?;
                    }
                }
                StmtControl::Break => break,
                StmtControl::Continue => {}
            }
            let Some(next) = value.checked_add(step) else {
                break;
            };
            value = next;
        }
        Ok(loop_result)
    }

    pub(crate) fn eval_while_loop(
        &mut self,
        condition: &HirExpr,
        body: &[HirStmt],
    ) -> Result<(), RuntimeError> {
        let mut iterations = 0_usize;
        loop {
            match self.eval_expr(condition)? {
                PineValue::Bool(true) => {}
                PineValue::Bool(false) | PineValue::Na => break,
                _ => break,
            }

            if iterations >= MAX_WHILE_ITERATIONS {
                return Err(RuntimeError {
                    message: format!(
                        "while loop exceeded maximum iteration count of {MAX_WHILE_ITERATIONS}"
                    ),
                });
            }
            iterations += 1;

            for statement in body {
                match self.eval_stmt(statement)? {
                    StmtControl::None => {}
                    StmtControl::Break => return Ok(()),
                    StmtControl::Continue => break,
                }
            }
        }

        Ok(())
    }
}
