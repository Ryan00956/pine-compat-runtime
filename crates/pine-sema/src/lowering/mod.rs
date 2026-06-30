use crate::prelude::*;

mod budget;
mod function_returns;
mod inline_calls;

pub(crate) fn prepend_block_statements(mut prefix: Vec<HirStmt>, expr: HirExpr) -> HirExpr {
    match expr.kind {
        HirExprKind::Block { statements, result } => {
            prefix.extend(statements);
            HirExpr {
                kind: HirExprKind::Block {
                    statements: prefix,
                    result,
                },
                pine_type: expr.pine_type,
                series_id: expr.series_id,
            }
        }
        _ => HirExpr {
            pine_type: expr.pine_type,
            series_id: expr.series_id,
            kind: HirExprKind::Block {
                statements: prefix,
                result: Box::new(expr),
            },
        },
    }
}

pub(crate) fn lower_literal(literal: &Literal) -> HirLiteral {
    match literal {
        Literal::Int(value) => HirLiteral::Int(*value),
        Literal::Float(value) => HirLiteral::Float(*value),
        Literal::Bool(value) => HirLiteral::Bool(*value),
        Literal::String(value) => HirLiteral::String(value.clone()),
        Literal::ColorHex(value) => HirLiteral::ColorHex(value.clone()),
    }
}
pub(crate) fn lower_unary_op(op: UnaryOp) -> HirUnaryOp {
    match op {
        UnaryOp::Plus => HirUnaryOp::Plus,
        UnaryOp::Minus => HirUnaryOp::Minus,
        UnaryOp::Not => HirUnaryOp::Not,
    }
}
pub(crate) fn lower_binary_op(op: BinaryOp) -> HirBinaryOp {
    match op {
        BinaryOp::Add => HirBinaryOp::Add,
        BinaryOp::Sub => HirBinaryOp::Sub,
        BinaryOp::Mul => HirBinaryOp::Mul,
        BinaryOp::Div => HirBinaryOp::Div,
        BinaryOp::Mod => HirBinaryOp::Mod,
        BinaryOp::Eq => HirBinaryOp::Eq,
        BinaryOp::NotEq => HirBinaryOp::NotEq,
        BinaryOp::Gt => HirBinaryOp::Gt,
        BinaryOp::Gte => HirBinaryOp::Gte,
        BinaryOp::Lt => HirBinaryOp::Lt,
        BinaryOp::Lte => HirBinaryOp::Lte,
        BinaryOp::And => HirBinaryOp::And,
        BinaryOp::Or => HirBinaryOp::Or,
    }
}
pub(crate) fn constant_history_offset(expr: &Expr) -> Option<u32> {
    match expr.kind {
        ExprKind::Literal(Literal::Int(value)) if value >= 0 => Some(value as u32),
        _ => None,
    }
}

fn sort_field_index_expr(index: usize) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Literal(HirLiteral::Int(index as i64)),
        pine_type: PineType::new(Qualifier::Const, ValueKind::Int),
        series_id: None,
    }
}

impl Analyzer {
    pub(crate) fn lower_program(&mut self, program: &Program) -> Option<HirProgram> {
        let mut statements = Vec::new();
        for statement in &program.statements {
            if matches!(
                statement.kind,
                StmtKind::Function { .. }
                    | StmtKind::Import(_)
                    | StmtKind::Library(_)
                    | StmtKind::Export(_)
                    | StmtKind::UserType(_)
                    | StmtKind::Method(_)
                    | StmtKind::Unsupported { .. }
            ) {
                continue;
            }
            statements.push(self.lower_stmt(statement)?);
        }
        if self.has_errors() {
            return None;
        }

        let symbols = self.lower_symbols();
        let history = infer_history_requirements(&statements, &symbols);
        let max_bars_back = infer_max_bars_back(&statements);
        let series_max_bars_back = infer_series_max_bars_back(&statements);
        let script_mode = self
            .script_declaration
            .map_or(ScriptMode::Indicator, |(mode, _)| mode);
        Some(HirProgram {
            script_mode,
            strategy_settings: self.strategy_settings,
            drawing_settings: self.drawing_settings,
            symbols,
            statements,
            next_series_id: self.next_series_id,
            next_call_site_id: self.next_call_site_id,
            next_var_slot_id: self.next_var_slot_id,
            max_bars_back,
            series_max_bars_back,
            history: history.program,
            series_history: history.series,
        })
    }

    pub(crate) fn lower_symbols(&self) -> Vec<HirSymbol> {
        self.scope.lower_symbols()
    }

    pub(crate) fn bind_symbol(&mut self, name: &str, span: Span, symbol: SymbolInfo) {
        self.bindings.insert(binding_key(name, span), symbol);
    }

    pub(crate) fn bound_symbol(&self, name: &str, span: Span) -> Option<SymbolInfo> {
        let symbol = self.bindings.get(&binding_key(name, span)).copied()?;
        self.lower_symbol_overrides
            .iter()
            .rev()
            .find_map(|overrides| overrides.get(&symbol.id).copied())
            .or(Some(symbol))
    }

    pub(crate) fn has_lower_symbol_override(&self, symbol_id: SymbolId) -> bool {
        self.lower_symbol_overrides
            .iter()
            .rev()
            .any(|overrides| overrides.contains_key(&symbol_id))
    }

    pub(crate) fn lower_decl_symbol(&mut self, name: &str, span: Span) -> Option<SymbolInfo> {
        let symbol = self.bindings.get(&binding_key(name, span)).copied()?;
        if self.lower_symbol_overrides.is_empty() || self.scope.contains_lower_symbol(symbol.id) {
            return Some(symbol);
        }
        if let Some(existing) = self
            .lower_symbol_overrides
            .iter()
            .rev()
            .find_map(|overrides| overrides.get(&symbol.id).copied())
        {
            return Some(existing);
        }
        let fresh = self.fresh_lower_symbol(name, symbol);
        self.lower_symbol_overrides
            .last_mut()
            .expect("override scope is active")
            .insert(symbol.id, fresh);
        Some(fresh)
    }

    pub(crate) fn lower_stmt(&mut self, statement: &pine_syntax::Stmt) -> Option<HirStmt> {
        self.lower_stmt_with_params(statement, &HashMap::new(), &HashMap::new())
    }

    pub(crate) fn lower_stmt_with_params(
        &mut self,
        statement: &pine_syntax::Stmt,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirStmt> {
        if !self.record_lowering_node(statement.span) {
            return None;
        }

        let kind = match &statement.kind {
            StmtKind::Expr(expr) => {
                HirStmtKind::Expr(self.lower_expr_with_params(expr, param_exprs, param_types)?)
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => HirStmtKind::If {
                condition: self.lower_expr_with_params(condition, param_exprs, param_types)?,
                then_branch: then_branch
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
                else_branch: else_branch
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
            },
            StmtKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => HirStmtKind::For {
                counter: self.lower_decl_symbol(counter, statement.span)?.id,
                from: self.lower_expr_with_params(from, param_exprs, param_types)?,
                to: self.lower_expr_with_params(to, param_exprs, param_types)?,
                step: match step {
                    Some(step) => {
                        Some(self.lower_expr_with_params(step, param_exprs, param_types)?)
                    }
                    None => None,
                },
                body: body
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
            },
            StmtKind::ForIn {
                index,
                value,
                iterable,
                body,
            } => HirStmtKind::ForIn {
                index: match index {
                    Some(index) => Some(self.lower_decl_symbol(index, statement.span)?.id),
                    None => None,
                },
                value: self.lower_decl_symbol(value, statement.span)?.id,
                iterable: self.lower_expr_with_params(iterable, param_exprs, param_types)?,
                body: body
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
            },
            StmtKind::While { condition, body } => HirStmtKind::While {
                condition: self.lower_expr_with_params(condition, param_exprs, param_types)?,
                body: body
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<_>>()?,
            },
            StmtKind::Break => HirStmtKind::Break,
            StmtKind::Continue => HirStmtKind::Continue,
            StmtKind::Decl { name, value, .. } => {
                let symbol = self.lower_decl_symbol(name, statement.span)?;
                if let Some(type_name) = self.user_type_name_of_expr_with_params(value, param_exprs)
                {
                    self.mark_symbol_id_user_type(symbol.id, type_name);
                }
                HirStmtKind::Decl {
                    symbol: symbol.id,
                    value: self.lower_expr_with_params(value, param_exprs, param_types)?,
                }
            }
            StmtKind::Reassign { name, value } => HirStmtKind::Reassign {
                symbol: self.bound_symbol(name, statement.span)?.id,
                value: self.lower_expr_with_params(value, param_exprs, param_types)?,
            },
            StmtKind::FieldReassign {
                receiver,
                field,
                value,
            } => {
                let parts = vec![receiver.clone(), field.clone()];
                let access = self
                    .chart_point_field_access_for_lowering(&parts, statement.span)
                    .map(|access| (access.receiver, access.index))
                    .or_else(|| {
                        self.user_type_field_access_for_lowering(&parts, statement.span)
                            .and_then(|access| {
                                access
                                    .fields
                                    .first()
                                    .map(|field| (access.receiver, field.index))
                            })
                    })?;
                HirStmtKind::FieldReassign {
                    symbol: self.bound_symbol(&access.0, statement.span)?.id,
                    field_index: access.1,
                    value: self.lower_expr_with_params(value, param_exprs, param_types)?,
                }
            }
            StmtKind::TupleDecl { names, value } => HirStmtKind::TupleDecl {
                symbols: names
                    .iter()
                    .map(|name| {
                        self.lower_decl_symbol(name, statement.span)
                            .map(|symbol| symbol.id)
                    })
                    .collect::<Option<_>>()?,
                value: self.lower_expr_with_params(value, param_exprs, param_types)?,
            },
            StmtKind::Function { .. }
            | StmtKind::Import(_)
            | StmtKind::Library(_)
            | StmtKind::Export(_)
            | StmtKind::UserType(_)
            | StmtKind::Method(_) => return None,
            StmtKind::Unsupported { .. } => return None,
        };

        Some(HirStmt { kind })
    }

    pub(crate) fn lower_expr_with_params(
        &mut self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        if !self.record_lowering_node(expr.span) {
            return None;
        }

        if let ExprKind::Identifier(name) = &expr.kind
            && let Some(param_expr) = param_exprs.get(name)
            && self
                .bindings
                .get(&binding_key(name, expr.span))
                .is_none_or(|symbol| !self.has_lower_symbol_override(symbol.id))
        {
            return Some(param_expr.clone());
        }

        let pine_type = self.type_of_expr_with_params(expr, param_types)?;
        let series_id = if (pine_type.qualifier == Qualifier::Series
            || is_collection_kind(pine_type.kind))
            && pine_type.kind != ValueKind::Tuple
        {
            match &expr.kind {
                ExprKind::Identifier(name) => self
                    .bound_symbol(name, expr.span)
                    .and_then(|symbol| symbol.series_id),
                _ => Some(self.alloc_series()),
            }
        } else {
            None
        };

        let kind = match &expr.kind {
            ExprKind::Literal(literal) => HirExprKind::Literal(lower_literal(literal)),
            ExprKind::Identifier(name) => {
                HirExprKind::Symbol(self.bound_symbol(name, expr.span)?.id)
            }
            ExprKind::QualifiedName(parts) => {
                if let Some(field) = self
                    .type_of_bound_chart_point_field_access(parts, expr.span)
                    .or_else(|| self.type_of_chart_point_field_access(parts))
                {
                    let access = self.chart_point_field_access_for_lowering(parts, expr.span)?;
                    let receiver_symbol = self
                        .bound_symbol(&access.receiver, expr.span)
                        .or_else(|| self.scope.resolve(&access.receiver))?;
                    return Some(HirExpr {
                        pine_type: field,
                        series_id,
                        kind: HirExprKind::FieldAccess {
                            value: Box::new(param_exprs.get(&access.receiver).cloned().unwrap_or(
                                HirExpr {
                                    kind: HirExprKind::Symbol(receiver_symbol.id),
                                    pine_type: receiver_symbol.pine_type,
                                    series_id: receiver_symbol.series_id,
                                },
                            )),
                            index: access.index,
                        },
                    });
                }
                if let Some(field) = self
                    .type_of_bound_user_type_field_access(parts, expr.span)
                    .or_else(|| self.type_of_user_type_field_access(parts))
                {
                    let access = self.user_type_field_access_for_lowering(parts, expr.span)?;
                    let receiver_symbol = self
                        .bound_symbol(&access.receiver, expr.span)
                        .or_else(|| self.scope.resolve(&access.receiver))?;
                    let mut value = param_exprs
                        .get(&access.receiver)
                        .cloned()
                        .unwrap_or(HirExpr {
                            kind: HirExprKind::Symbol(receiver_symbol.id),
                            pine_type: receiver_symbol.pine_type,
                            series_id: receiver_symbol.series_id,
                        });
                    let last_index = access.fields.len().saturating_sub(1);
                    for (index, field_access) in access.fields.iter().enumerate() {
                        value = HirExpr {
                            pine_type: field_access.pine_type,
                            series_id: (index == last_index).then_some(series_id).flatten(),
                            kind: HirExprKind::FieldAccess {
                                value: Box::new(value),
                                index: field_access.index,
                            },
                        };
                    }
                    debug_assert_eq!(value.pine_type, field);
                    return Some(value);
                }
                HirExprKind::Builtin(parts.join("."))
            }
            ExprKind::Unary { op, expr } => HirExprKind::Unary {
                op: lower_unary_op(*op),
                expr: Box::new(self.lower_expr_with_params(expr, param_exprs, param_types)?),
            },
            ExprKind::Binary { op, left, right } => HirExprKind::Binary {
                op: lower_binary_op(*op),
                left: Box::new(self.lower_expr_with_params(left, param_exprs, param_types)?),
                right: Box::new(self.lower_expr_with_params(right, param_exprs, param_types)?),
            },
            ExprKind::Ternary {
                condition,
                then_expr,
                else_expr,
            } => HirExprKind::Ternary {
                condition: Box::new(self.lower_expr_with_params(
                    condition,
                    param_exprs,
                    param_types,
                )?),
                then_expr: Box::new(self.lower_expr_with_params(
                    then_expr,
                    param_exprs,
                    param_types,
                )?),
                else_expr: Box::new(self.lower_expr_with_params(
                    else_expr,
                    param_exprs,
                    param_types,
                )?),
            },
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => HirExprKind::Ternary {
                condition: Box::new(self.lower_expr_with_params(
                    condition,
                    param_exprs,
                    param_types,
                )?),
                then_expr: Box::new(self.lower_expr_branch_return(
                    then_branch,
                    param_exprs,
                    param_types,
                )?),
                else_expr: Box::new(self.lower_expr_branch_return(
                    else_branch,
                    param_exprs,
                    param_types,
                )?),
            },
            ExprKind::Switch { selector, arms } => HirExprKind::Switch {
                selector: match selector {
                    Some(selector) => Some(Box::new(self.lower_expr_with_params(
                        selector,
                        param_exprs,
                        param_types,
                    )?)),
                    None => None,
                },
                arms: arms
                    .iter()
                    .map(|arm| {
                        Some(HirSwitchArm {
                            condition: match &arm.condition {
                                Some(condition) => Some(self.lower_expr_with_params(
                                    condition,
                                    param_exprs,
                                    param_types,
                                )?),
                                None => None,
                            },
                            result: self.lower_switch_arm_result_with_params(
                                &arm.result,
                                param_exprs,
                                param_types,
                            )?,
                        })
                    })
                    .collect::<Option<_>>()?,
            },
            ExprKind::For {
                counter,
                from,
                to,
                step,
                body,
            } => {
                let (last, prefix) = body.split_last()?;
                let StmtKind::Expr(result) = &last.kind else {
                    return None;
                };
                HirExprKind::For {
                    counter: self.lower_decl_symbol(counter, expr.span)?.id,
                    from: Box::new(self.lower_expr_with_params(from, param_exprs, param_types)?),
                    to: Box::new(self.lower_expr_with_params(to, param_exprs, param_types)?),
                    step: match step {
                        Some(step) => Some(Box::new(self.lower_expr_with_params(
                            step,
                            param_exprs,
                            param_types,
                        )?)),
                        None => None,
                    },
                    statements: prefix
                        .iter()
                        .map(|statement| {
                            self.lower_stmt_with_params(statement, param_exprs, param_types)
                        })
                        .collect::<Option<_>>()?,
                    result: Box::new(self.lower_expr_with_params(
                        result,
                        param_exprs,
                        param_types,
                    )?),
                }
            }
            ExprKind::While { condition, body } => {
                let (last, prefix) = body.split_last()?;
                let StmtKind::Expr(result) = &last.kind else {
                    return None;
                };
                HirExprKind::While {
                    condition: Box::new(self.lower_expr_with_params(
                        condition,
                        param_exprs,
                        param_types,
                    )?),
                    statements: prefix
                        .iter()
                        .map(|statement| {
                            self.lower_stmt_with_params(statement, param_exprs, param_types)
                        })
                        .collect::<Option<_>>()?,
                    result: Box::new(self.lower_expr_with_params(
                        result,
                        param_exprs,
                        param_types,
                    )?),
                }
            }
            ExprKind::Tuple(items) => HirExprKind::Tuple(
                items
                    .iter()
                    .map(|item| self.lower_expr_with_params(item, param_exprs, param_types))
                    .collect::<Option<_>>()?,
            ),
            ExprKind::Call { callee, args } => {
                let name = expr_name(callee)?;
                if let Some(constructor) = self
                    .user_type_constructor_for_lowering(&name, args, param_types)
                    .or_else(|| {
                        self.imported_user_type_constructor_for_lowering(&name, args, param_types)
                    })
                {
                    return Some(HirExpr {
                        pine_type,
                        series_id,
                        kind: HirExprKind::UserTypeConstruct {
                            identity: HirUserTypeIdentity {
                                source_id: constructor.identity.source_id.get(),
                                type_name: constructor.identity.name,
                            },
                            fields: constructor
                                .field_args
                                .iter()
                                .map(|arg| {
                                    self.lower_expr_with_params(arg, param_exprs, param_types)
                                })
                                .collect::<Option<_>>()?,
                        },
                    });
                }
                if name == "array.from"
                    && pine_type.kind == ValueKind::UserTypeArray
                    && let Some(type_name) = self.expr_user_type_array_name(expr)
                {
                    return Some(HirExpr {
                        pine_type,
                        series_id,
                        kind: HirExprKind::UserTypeArrayConstruct {
                            type_name,
                            elements: args
                                .iter()
                                .map(|arg| {
                                    self.lower_expr_with_params(
                                        &arg.value,
                                        param_exprs,
                                        param_types,
                                    )
                                })
                                .collect::<Option<_>>()?,
                        },
                    });
                }
                if let Some((receiver_name, method_name)) = method_call_parts(callee)
                    && self
                        .bound_symbol(receiver_name, callee.span)
                        .and_then(|symbol| self.symbol_user_types.get(&symbol.id))
                        .is_some()
                {
                    return self.lower_user_method_call(
                        receiver_name,
                        method_name,
                        callee.span,
                        args,
                        param_exprs,
                        param_types,
                    );
                }
                if self.functions.contains_key(&name) {
                    return self.lower_udf_call(&name, expr.span, args, param_exprs, param_types);
                }
                if pine_builtins::get_phase_1_builtin(&name).is_none()
                    && let Some((receiver_name, method_name)) = method_call_parts(callee)
                    && let Some(builtin_name) = param_types
                        .get(receiver_name)
                        .map(|pine_type| pine_type.kind)
                        .or_else(|| {
                            self.bound_symbol(receiver_name, callee.span)
                                .map(|symbol| symbol.pine_type.kind)
                        })
                        .and_then(|receiver_kind| {
                            drawing_method_builtin_name(receiver_kind, method_name).or_else(|| {
                                matrix_method_builtin_name(receiver_kind, method_name)
                                    .map(ToOwned::to_owned)
                            })
                        })
                        .or_else(|| array_method_builtin_name(method_name).map(ToOwned::to_owned))
                {
                    let receiver_arg = receiver_call_arg(receiver_name, callee.span);
                    let mut method_args = Vec::with_capacity(args.len() + 1);
                    method_args.push(receiver_arg);
                    method_args.extend(args.iter().cloned());
                    let lowered_args = self.lower_builtin_call_args(
                        &builtin_name,
                        &method_args,
                        param_exprs,
                        param_types,
                    )?;
                    return Some(HirExpr {
                        pine_type,
                        series_id,
                        kind: HirExprKind::Call {
                            callee: builtin_name,
                            call_site_id: self.alloc_call_site(),
                            args: lowered_args,
                        },
                    });
                }
                let call_site_id = self.alloc_call_site();
                let lowered_args =
                    self.lower_builtin_call_args(&name, args, param_exprs, param_types)?;
                HirExprKind::Call {
                    callee: name,
                    call_site_id,
                    args: lowered_args,
                }
            }
            ExprKind::History { expr, offset } => {
                let offset = match constant_history_offset(offset) {
                    Some(offset) => HirHistoryOffset::Constant(offset),
                    None => HirHistoryOffset::Dynamic(Box::new(self.lower_expr_with_params(
                        offset,
                        param_exprs,
                        param_types,
                    )?)),
                };
                HirExprKind::History {
                    expr: Box::new(self.lower_expr_with_params(expr, param_exprs, param_types)?),
                    offset,
                }
            }
        };

        Some(HirExpr {
            kind,
            pine_type,
            series_id,
        })
    }

    fn lower_builtin_call_args(
        &mut self,
        builtin_name: &str,
        args: &[CallArg],
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<Vec<HirCallArg>> {
        let sort_field_index = matches!(builtin_name, "array.sort" | "array.sort_indices")
            .then(|| self.user_type_array_sort_field_index(args))
            .flatten();
        args.iter()
            .enumerate()
            .map(|(index, arg)| {
                let value = if index == 2 {
                    sort_field_index.map(sort_field_index_expr)
                } else {
                    None
                }
                .or_else(|| self.lower_expr_with_params(&arg.value, param_exprs, param_types))?;
                Some(HirCallArg {
                    name: arg.name.clone(),
                    value,
                })
            })
            .collect()
    }

    fn user_type_name_of_expr_with_params(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
    ) -> Option<String> {
        self.user_type_name_of_expr_with_params_and_aliases(expr, param_exprs, &HashMap::new())
    }

    fn user_type_name_of_expr_with_params_and_aliases(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
        aliases: &HashMap<String, String>,
    ) -> Option<String> {
        if let Some(type_name) = self.user_type_name_of_expr(expr) {
            return Some(type_name);
        }
        if let ExprKind::Ternary {
            then_expr,
            else_expr,
            ..
        } = &expr.kind
        {
            return match (
                self.user_type_name_of_expr_with_params_and_aliases(
                    then_expr,
                    param_exprs,
                    aliases,
                ),
                self.user_type_name_of_expr_with_params_and_aliases(
                    else_expr,
                    param_exprs,
                    aliases,
                ),
            ) {
                (Some(then_name), Some(else_name)) if then_name == else_name => Some(then_name),
                _ => None,
            };
        }
        if let ExprKind::Switch { arms, .. } = &expr.kind {
            let mut resolved_type_name = None;
            for arm in arms {
                match self.user_type_name_of_switch_arm_result_with_params_and_aliases(
                    &arm.result,
                    param_exprs,
                    aliases,
                ) {
                    Some(type_name) => match &resolved_type_name {
                        Some(resolved) if resolved != &type_name => return None,
                        Some(_) => {}
                        None => resolved_type_name = Some(type_name),
                    },
                    None => return None,
                }
            }
            return resolved_type_name;
        }
        if let ExprKind::For { body, .. } = &expr.kind {
            let (last, prefix) = body.split_last()?;
            let StmtKind::Expr(result) = &last.kind else {
                return None;
            };
            let aliases = self.local_user_type_param_aliases(prefix, param_exprs, aliases);
            return self.user_type_name_of_expr_with_params_and_aliases(
                result,
                param_exprs,
                &aliases,
            );
        }
        let name = match &expr.kind {
            ExprKind::Identifier(name) => name,
            ExprKind::QualifiedName(parts) if parts.len() == 1 => &parts[0],
            _ => return None,
        };
        if let Some(type_name) = aliases.get(name) {
            return Some(type_name.clone());
        }
        let HirExprKind::Symbol(symbol_id) = param_exprs.get(name)?.kind else {
            return None;
        };
        self.symbol_user_types.get(&symbol_id).cloned()
    }

    fn local_user_type_param_aliases(
        &self,
        prefix: &[Stmt],
        param_exprs: &HashMap<String, HirExpr>,
        outer_aliases: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut aliases = outer_aliases.clone();
        for statement in prefix {
            if let StmtKind::Decl { name, value, .. } = &statement.kind
                && let Some(type_name) = self.user_type_name_of_expr_with_params_and_aliases(
                    value,
                    param_exprs,
                    &aliases,
                )
            {
                aliases.insert(name.clone(), type_name);
            }
        }
        aliases
    }

    fn user_type_name_of_switch_arm_result_with_params_and_aliases(
        &self,
        result: &SwitchArmResult,
        param_exprs: &HashMap<String, HirExpr>,
        aliases: &HashMap<String, String>,
    ) -> Option<String> {
        match result {
            SwitchArmResult::Expr(expr) => {
                self.user_type_name_of_expr_with_params_and_aliases(expr, param_exprs, aliases)
            }
            SwitchArmResult::Block(statements) => {
                let (last, prefix) = statements.split_last()?;
                let StmtKind::Expr(result) = &last.kind else {
                    return None;
                };
                let aliases = self.local_user_type_param_aliases(prefix, param_exprs, aliases);
                self.user_type_name_of_expr_with_params_and_aliases(result, param_exprs, &aliases)
            }
        }
    }
}
