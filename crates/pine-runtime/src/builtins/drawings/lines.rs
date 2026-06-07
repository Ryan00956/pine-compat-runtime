use pine_ir::{HirCallArg, HirExpr};

use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(super) fn eval_line_new(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let x1 = self.eval_required_line_arg(args, 0, "x1")?;
        let y1 = self.eval_required_line_arg(args, 1, "y1")?;
        let x2 = self.eval_required_line_arg(args, 2, "x2")?;
        let y2 = self.eval_required_line_arg(args, 3, "y2")?;
        let _xloc = self.eval_line_option(args, 4, "xloc", "xloc.bar_index")?;
        let extend = self.eval_line_option(args, 5, "extend", "extend.none")?;
        let color = self.eval_line_option_value(args, 6, "color", PineValue::Na)?;
        let style = self.eval_line_option(args, 7, "style", "line.style_solid")?;
        let width = self.eval_line_option_value(args, 8, "width", PineValue::Int(1))?;
        let _force_overlay =
            self.eval_line_option_value(args, 9, "force_overlay", PineValue::Bool(false))?;
        if self.lines.len() >= MAX_LINES {
            return Err(RuntimeError {
                message: format!("line count cannot exceed {MAX_LINES}"),
            });
        }
        let id = self.next_line_id;
        self.next_line_id = self
            .next_line_id
            .checked_add(1)
            .ok_or_else(|| RuntimeError {
                message: "line id limit exceeded".to_owned(),
            })?;
        self.lines.push(LineOutput {
            id,
            snapshots: vec![LineSnapshot {
                bar_index: self.bars,
                exists: true,
                x1,
                y1,
                x2,
                y2,
                color,
                width,
                style,
                extend,
            }],
        });
        Ok(PineValue::Line(id))
    }

    pub(super) fn eval_line_set_x1(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let x = self.eval_required_line_arg(args, 1, "x")?;
        self.mutate_line(id, |snapshot| {
            snapshot.x1 = x;
        })
    }

    pub(super) fn eval_line_set_y1(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let y = self.eval_required_line_arg(args, 1, "y")?;
        self.mutate_line(id, |snapshot| {
            snapshot.y1 = y;
        })
    }

    pub(super) fn eval_line_set_xy1(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let x = self.eval_required_line_arg(args, 1, "x")?;
        let y = self.eval_required_line_arg(args, 2, "y")?;
        self.mutate_line(id, |snapshot| {
            snapshot.x1 = x;
            snapshot.y1 = y;
        })
    }

    pub(super) fn eval_line_set_x2(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let x = self.eval_required_line_arg(args, 1, "x")?;
        self.mutate_line(id, |snapshot| {
            snapshot.x2 = x;
        })
    }

    pub(super) fn eval_line_set_y2(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let y = self.eval_required_line_arg(args, 1, "y")?;
        self.mutate_line(id, |snapshot| {
            snapshot.y2 = y;
        })
    }

    pub(super) fn eval_line_set_xy2(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let x = self.eval_required_line_arg(args, 1, "x")?;
        let y = self.eval_required_line_arg(args, 2, "y")?;
        self.mutate_line(id, |snapshot| {
            snapshot.x2 = x;
            snapshot.y2 = y;
        })
    }

    pub(super) fn eval_line_set_color(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let color = self.eval_required_line_arg(args, 1, "color")?;
        self.mutate_line(id, |snapshot| {
            snapshot.color = color;
        })
    }

    pub(super) fn eval_line_set_width(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let width = self.eval_required_line_arg(args, 1, "width")?;
        self.mutate_line(id, |snapshot| {
            snapshot.width = width;
        })
    }

    pub(super) fn eval_line_set_style(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let style = self.eval_required_line_arg(args, 1, "style")?;
        self.mutate_line(id, |snapshot| {
            snapshot.style = style;
        })
    }

    pub(super) fn eval_line_set_extend(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let extend = self.eval_required_line_arg(args, 1, "extend")?;
        self.mutate_line(id, |snapshot| {
            snapshot.extend = extend;
        })
    }

    pub(super) fn eval_line_delete(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let Some(id) = id else {
            return Ok(PineValue::Void);
        };
        let Some(line) = self.lines.iter_mut().find(|line| line.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid line id `{id}`"),
            });
        };
        let Some(latest) = line.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("line `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Void);
        }
        let mut next = latest;
        next.bar_index = self.bars;
        next.exists = false;
        line.snapshots.push(next);
        Ok(PineValue::Void)
    }

    pub(super) fn eval_line_copy(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_id_arg(args)?;
        let Some(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(line) = self.lines.iter().find(|line| line.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid line id `{id}`"),
            });
        };
        let Some(latest) = line.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("line `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Na);
        }
        if self.lines.len() >= MAX_LINES {
            return Err(RuntimeError {
                message: format!("line count cannot exceed {MAX_LINES}"),
            });
        }
        let copied_id = self.next_line_id;
        self.next_line_id = self
            .next_line_id
            .checked_add(1)
            .ok_or_else(|| RuntimeError {
                message: "line id limit exceeded".to_owned(),
            })?;
        let mut copied = latest;
        copied.bar_index = self.bars;
        self.lines.push(LineOutput {
            id: copied_id,
            snapshots: vec![copied],
        });
        Ok(PineValue::Line(copied_id))
    }

    pub(super) fn eval_line_get_x1(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_line_get(args, "line.get_x1", |snapshot| snapshot.x1.clone())
    }

    pub(super) fn eval_line_get_y1(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_line_get(args, "line.get_y1", |snapshot| snapshot.y1.clone())
    }

    pub(super) fn eval_line_get_x2(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_line_get(args, "line.get_x2", |snapshot| snapshot.x2.clone())
    }

    pub(super) fn eval_line_get_y2(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_line_get(args, "line.get_y2", |snapshot| snapshot.y2.clone())
    }

    fn eval_line_id_arg(&mut self, args: &[HirCallArg]) -> Result<Option<u32>, RuntimeError> {
        let Some(id_arg) = line_call_arg_expr(args, 0, "id") else {
            return Err(RuntimeError {
                message: "line mutation missing id argument".to_owned(),
            });
        };
        match self.eval_expr(id_arg)? {
            PineValue::Line(id) => Ok(Some(id)),
            PineValue::Na => Ok(None),
            value => Err(RuntimeError {
                message: format!("line mutation expected line id, got {value:?}"),
            }),
        }
    }

    fn eval_required_line_arg(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
    ) -> Result<PineValue, RuntimeError> {
        let Some(arg) = line_call_arg_expr(args, index, name) else {
            return Err(RuntimeError {
                message: format!("line.new missing {name} argument"),
            });
        };
        self.eval_expr(arg)
    }

    fn eval_line_option(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
        default: &str,
    ) -> Result<PineValue, RuntimeError> {
        self.eval_line_option_value(args, index, name, PineValue::String(default.to_owned()))
    }

    fn eval_line_option_value(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
        default: PineValue,
    ) -> Result<PineValue, RuntimeError> {
        match line_call_arg_expr(args, index, name) {
            Some(expr) => self.eval_expr(expr),
            None => Ok(default),
        }
    }

    fn eval_line_get(
        &mut self,
        args: &[HirCallArg],
        function_name: &str,
        get_value: impl FnOnce(&LineSnapshot) -> PineValue,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_line_get_id_arg(args, function_name)?;
        let Some(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(line) = self.lines.iter().find(|line| line.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid line id `{id}`"),
            });
        };
        let Some(latest) = line.snapshots.last() else {
            return Err(RuntimeError {
                message: format!("line `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Na);
        }
        Ok(get_value(latest))
    }

    fn eval_line_get_id_arg(
        &mut self,
        args: &[HirCallArg],
        function_name: &str,
    ) -> Result<Option<u32>, RuntimeError> {
        let Some(id_arg) = line_call_arg_expr(args, 0, "id") else {
            return Err(RuntimeError {
                message: format!("{function_name} missing id argument"),
            });
        };
        match self.eval_expr(id_arg)? {
            PineValue::Line(id) => Ok(Some(id)),
            PineValue::Na => Ok(None),
            value => Err(RuntimeError {
                message: format!("{function_name} expected line id, got {value:?}"),
            }),
        }
    }

    fn mutate_line(
        &mut self,
        id: Option<u32>,
        mutate: impl FnOnce(&mut LineSnapshot),
    ) -> Result<PineValue, RuntimeError> {
        let Some(id) = id else {
            return Ok(PineValue::Void);
        };
        let Some(line) = self.lines.iter_mut().find(|line| line.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid line id `{id}`"),
            });
        };
        let Some(latest) = line.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("line `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Void);
        }
        let mut next = latest.clone();
        mutate(&mut next);
        if next != latest {
            next.bar_index = self.bars;
            line.snapshots.push(next);
        }
        Ok(PineValue::Void)
    }
}

fn line_call_arg_expr<'a>(args: &'a [HirCallArg], index: usize, name: &str) -> Option<&'a HirExpr> {
    args.iter()
        .find(|arg| arg.name.as_deref() == Some(name))
        .or_else(|| args.get(index).filter(|arg| arg.name.is_none()))
        .map(|arg| &arg.value)
}
