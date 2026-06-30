use std::collections::HashMap;

use crate::prelude::*;

use super::prepend_block_statements;

impl Analyzer {
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
                self.mark_symbol_id_user_type(symbol.id, type_name);
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
        self.mark_symbol_id_user_type(receiver_temp.id, method.receiver_type.clone());
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
            let arg_user_type =
                self.user_type_name_of_expr_with_params(&arg.value, outer_param_exprs);
            let arg_expr =
                self.lower_expr_with_params(&arg.value, outer_param_exprs, outer_param_types)?;
            let arg_type = self.type_of_expr_with_params(&arg.value, outer_param_types)?;
            resolved_args[param_index] = Some((arg_expr, arg_type, arg_user_type));
        }
        for (param, resolved_arg) in method.params.iter().zip(resolved_args) {
            let (arg_expr, arg_type, arg_user_type) = resolved_arg?;
            if !self.record_lowering_temp_symbol(receiver_span) {
                return None;
            }
            let symbol = self.fresh_temp_symbol(&format!("{method_name}.{}", param.name), arg_type);
            if let Some(type_name) = arg_user_type {
                self.mark_symbol_id_user_type(symbol.id, type_name);
            }
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
}
