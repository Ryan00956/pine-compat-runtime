use crate::prelude::*;

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
        let script_mode = self
            .script_declaration
            .map_or(ScriptMode::Indicator, |(mode, _)| mode);
        Some(HirProgram {
            script_mode,
            strategy_settings: self.strategy_settings,
            symbols,
            statements,
            next_series_id: self.next_series_id,
            next_call_site_id: self.next_call_site_id,
            next_var_slot_id: self.next_var_slot_id,
            max_bars_back,
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
                    self.symbol_user_types.insert(symbol.id, type_name);
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
        let series_id =
            if pine_type.qualifier == Qualifier::Series && pine_type.kind != ValueKind::Tuple {
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
                    .type_of_bound_user_type_field_access(parts, expr.span)
                    .or_else(|| self.type_of_user_type_field_access(parts))
                {
                    let access = self.user_type_field_access_for_lowering(parts, expr.span)?;
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
                            result: self.lower_expr_with_params(
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
            ExprKind::Tuple(items) => HirExprKind::Tuple(
                items
                    .iter()
                    .map(|item| self.lower_expr_with_params(item, param_exprs, param_types))
                    .collect::<Option<_>>()?,
            ),
            ExprKind::Call { callee, args } => {
                let name = expr_name(callee)?;
                if let Some(constructor) = self.user_type_constructor_for_lowering(&name, args) {
                    return Some(HirExpr {
                        pine_type,
                        series_id,
                        kind: HirExprKind::UserTypeConstruct {
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
                            drawing_method_builtin_name(receiver_kind, method_name)
                        })
                        .or_else(|| array_method_builtin_name(method_name).map(ToOwned::to_owned))
                {
                    let mut lowered_args = Vec::with_capacity(args.len() + 1);
                    let receiver_arg = receiver_call_arg(receiver_name, callee.span);
                    lowered_args.push(HirCallArg {
                        name: None,
                        value: self.lower_expr_with_params(
                            &receiver_arg.value,
                            param_exprs,
                            param_types,
                        )?,
                    });
                    lowered_args.extend(
                        args.iter()
                            .map(|arg| {
                                Some(HirCallArg {
                                    name: arg.name.clone(),
                                    value: self.lower_expr_with_params(
                                        &arg.value,
                                        param_exprs,
                                        param_types,
                                    )?,
                                })
                            })
                            .collect::<Option<Vec<_>>>()?,
                    );
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
                HirExprKind::Call {
                    callee: name,
                    call_site_id: self.alloc_call_site(),
                    args: args
                        .iter()
                        .map(|arg| {
                            Some(HirCallArg {
                                name: arg.name.clone(),
                                value: self.lower_expr_with_params(
                                    &arg.value,
                                    param_exprs,
                                    param_types,
                                )?,
                            })
                        })
                        .collect::<Option<_>>()?,
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

    pub(crate) fn lower_udf_call(
        &mut self,
        name: &str,
        span: Span,
        args: &[CallArg],
        outer_param_exprs: &HashMap<String, HirExpr>,
        outer_param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        let function = self.functions.get(name)?.clone();
        let arg_indices = resolve_udf_arg_indices(&function.params, args).ok()?;
        let mut resolved_args = vec![None; function.params.len()];
        for (arg, param_index) in args.iter().zip(arg_indices) {
            let arg_user_type =
                self.user_type_name_of_expr_with_params(&arg.value, outer_param_exprs);
            let arg_expr =
                self.lower_expr_with_params(&arg.value, outer_param_exprs, outer_param_types)?;
            let arg_type = self.type_of_expr_with_params(&arg.value, outer_param_types)?;
            resolved_args[param_index] = Some((arg_expr, arg_type, arg_user_type));
        }

        let mut param_exprs = HashMap::new();
        let mut param_types = HashMap::new();
        let mut arg_statements = Vec::new();
        for (param, resolved_arg) in function.params.iter().zip(resolved_args) {
            let (arg_expr, arg_type, arg_user_type) = resolved_arg?;
            if !self.record_lowering_temp_symbol(span) {
                return None;
            }
            let symbol = self.fresh_temp_symbol(&format!("{name}.{param}"), arg_type);
            if let Some(type_name) = arg_user_type {
                self.symbol_user_types.insert(symbol.id, type_name);
            }
            arg_statements.push(HirStmt {
                kind: HirStmtKind::Decl {
                    symbol: symbol.id,
                    value: arg_expr,
                },
            });
            param_exprs.insert(
                param.clone(),
                HirExpr {
                    kind: HirExprKind::Symbol(symbol.id),
                    pine_type: arg_type,
                    series_id: symbol.series_id,
                },
            );
            param_types.insert(param.clone(), arg_type);
        }
        if !self.enter_lowering_inline(span) {
            return None;
        }
        let body = self.lower_function_body(&function.body, &param_exprs, &param_types);
        self.exit_lowering_inline();
        let body = body?;
        Some(prepend_block_statements(arg_statements, body))
    }

    pub(crate) fn lower_user_method_call(
        &mut self,
        receiver_name: &str,
        method_name: &str,
        receiver_span: Span,
        args: &[CallArg],
        outer_param_exprs: &HashMap<String, HirExpr>,
        outer_param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        let receiver_symbol = self
            .bound_symbol(receiver_name, receiver_span)
            .or_else(|| self.scope.resolve(receiver_name))?;
        let receiver_type_name = self.symbol_user_types.get(&receiver_symbol.id)?.clone();
        let method = self
            .methods
            .get(&(receiver_type_name, method_name.to_owned()))?
            .clone();
        let param_names: Vec<_> = method
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let arg_indices = resolve_udf_arg_indices(&param_names, args).ok()?;

        let mut param_exprs = HashMap::new();
        let mut param_types = HashMap::new();
        let mut arg_statements = Vec::new();
        let receiver_expr = outer_param_exprs
            .get(receiver_name)
            .cloned()
            .unwrap_or(HirExpr {
                kind: HirExprKind::Symbol(receiver_symbol.id),
                pine_type: receiver_symbol.pine_type,
                series_id: receiver_symbol.series_id,
            });
        if !self.record_lowering_temp_symbol(receiver_span) {
            return None;
        }
        let receiver_temp = self.fresh_temp_symbol(
            &format!("{method_name}.{receiver_name}"),
            receiver_expr.pine_type,
        );
        arg_statements.push(HirStmt {
            kind: HirStmtKind::Decl {
                symbol: receiver_temp.id,
                value: receiver_expr,
            },
        });
        param_exprs.insert(
            method.receiver_name.clone(),
            HirExpr {
                kind: HirExprKind::Symbol(receiver_temp.id),
                pine_type: receiver_temp.pine_type,
                series_id: receiver_temp.series_id,
            },
        );
        param_types.insert(method.receiver_name.clone(), receiver_temp.pine_type);

        let mut resolved_args = vec![None; method.params.len()];
        for (arg, param_index) in args.iter().zip(arg_indices) {
            let arg_expr =
                self.lower_expr_with_params(&arg.value, outer_param_exprs, outer_param_types)?;
            let arg_type = self.type_of_expr_with_params(&arg.value, outer_param_types)?;
            resolved_args[param_index] = Some((arg_expr, arg_type));
        }
        for (param, resolved_arg) in method.params.iter().zip(resolved_args) {
            let (arg_expr, arg_type) = resolved_arg?;
            if !self.record_lowering_temp_symbol(receiver_span) {
                return None;
            }
            let symbol = self.fresh_temp_symbol(&format!("{method_name}.{}", param.name), arg_type);
            arg_statements.push(HirStmt {
                kind: HirStmtKind::Decl {
                    symbol: symbol.id,
                    value: arg_expr,
                },
            });
            param_exprs.insert(
                param.name.clone(),
                HirExpr {
                    kind: HirExprKind::Symbol(symbol.id),
                    pine_type: arg_type,
                    series_id: symbol.series_id,
                },
            );
            param_types.insert(param.name.clone(), arg_type);
        }
        if !self.enter_lowering_inline(receiver_span) {
            return None;
        }
        let body = self.lower_function_body(&method.body, &param_exprs, &param_types);
        self.exit_lowering_inline();
        let body = body?;
        Some(prepend_block_statements(arg_statements, body))
    }

    fn user_type_name_of_expr_with_params(
        &self,
        expr: &Expr,
        param_exprs: &HashMap<String, HirExpr>,
    ) -> Option<String> {
        if let Some(type_name) = self.user_type_name_of_expr(expr) {
            return Some(type_name);
        }
        let name = match &expr.kind {
            ExprKind::Identifier(name) => name,
            ExprKind::QualifiedName(parts) if parts.len() == 1 => &parts[0],
            _ => return None,
        };
        let HirExprKind::Symbol(symbol_id) = param_exprs.get(name)?.kind else {
            return None;
        };
        self.symbol_user_types.get(&symbol_id).cloned()
    }

    pub(crate) fn lower_function_body(
        &mut self,
        body: &FunctionBody,
        param_exprs: &HashMap<String, HirExpr>,
        param_types: &HashMap<String, PineType>,
    ) -> Option<HirExpr> {
        match body {
            FunctionBody::Expr(expr) => {
                self.lower_symbol_overrides.push(HashMap::new());
                let result = self.lower_expr_with_params(expr, param_exprs, param_types);
                self.lower_symbol_overrides.pop();
                result
            }
            FunctionBody::Block(statements) => {
                let (last, prefix) = statements.split_last()?;
                let StmtKind::Expr(result) = &last.kind else {
                    return None;
                };
                self.lower_symbol_overrides.push(HashMap::new());
                let lowered_statements = prefix
                    .iter()
                    .map(|statement| {
                        self.lower_stmt_with_params(statement, param_exprs, param_types)
                    })
                    .collect::<Option<Vec<_>>>();
                let result = lowered_statements.and_then(|statements| {
                    Some((
                        statements,
                        self.lower_expr_with_params(result, param_exprs, param_types)?,
                    ))
                });
                self.lower_symbol_overrides.pop();
                let (statements, result) = result?;
                Some(HirExpr {
                    pine_type: result.pine_type,
                    series_id: result.series_id,
                    kind: HirExprKind::Block {
                        statements,
                        result: Box::new(result),
                    },
                })
            }
        }
    }

    fn enter_lowering_inline(&mut self, span: Span) -> bool {
        if self.lowering_inline_depth >= self.lowering_limits.max_inline_depth {
            self.report_lowering_budget_exceeded("lowering inline call chain is too deep", span);
            return false;
        }

        self.lowering_inline_depth += 1;
        true
    }

    fn exit_lowering_inline(&mut self) {
        self.lowering_inline_depth = self.lowering_inline_depth.saturating_sub(1);
    }

    fn record_lowering_node(&mut self, span: Span) -> bool {
        if self.lowered_hir_nodes >= self.lowering_limits.max_hir_nodes {
            self.report_lowering_budget_exceeded("lowered HIR is too large", span);
            return false;
        }

        self.lowered_hir_nodes += 1;
        true
    }

    fn record_lowering_temp_symbol(&mut self, span: Span) -> bool {
        if self.lowered_temp_symbols >= self.lowering_limits.max_temp_symbols {
            self.report_lowering_budget_exceeded(
                "lowering generated too many temporary symbols",
                span,
            );
            return false;
        }

        self.lowered_temp_symbols += 1;
        true
    }

    fn report_lowering_budget_exceeded(&mut self, message: &str, span: Span) {
        if self.lowering_budget_reported {
            return;
        }

        self.lowering_budget_reported = true;
        self.diagnostics
            .push(Diagnostic::error("E_LOWERING_BUDGET", message, span));
    }
}
