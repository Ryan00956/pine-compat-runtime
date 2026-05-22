use pine_ir::HirCallArg;

use crate::builtins::args::call_arg_expr;
use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(super) fn eval_line_new(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let x1 = self.eval_required_line_arg(args, 0, "x1")?;
        let y1 = self.eval_required_line_arg(args, 1, "y1")?;
        let x2 = self.eval_required_line_arg(args, 2, "x2")?;
        let y2 = self.eval_required_line_arg(args, 3, "y2")?;
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
                color: PineValue::Na,
                width: PineValue::Int(1),
                style: PineValue::String("line.style_solid".to_owned()),
                extend: PineValue::String("extend.none".to_owned()),
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

    fn eval_line_id_arg(&mut self, args: &[HirCallArg]) -> Result<Option<u32>, RuntimeError> {
        let Some(id_arg) = call_arg_expr(args, 0, "id") else {
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
        let Some(arg) = call_arg_expr(args, index, name) else {
            return Err(RuntimeError {
                message: format!("line.new missing {name} argument"),
            });
        };
        self.eval_expr(arg)
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
