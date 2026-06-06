use pine_ir::HirCallArg;

use crate::builtins::args::call_arg_expr;
use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(super) fn eval_box_new(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let left = self.eval_required_box_arg(args, 0, "left")?;
        let top = self.eval_required_box_arg(args, 1, "top")?;
        let right = self.eval_required_box_arg(args, 2, "right")?;
        let bottom = self.eval_required_box_arg(args, 3, "bottom")?;
        if self.boxes.len() >= MAX_BOXES {
            return Err(RuntimeError {
                message: format!("box count cannot exceed {MAX_BOXES}"),
            });
        }
        let id = self.next_box_id;
        self.next_box_id = self
            .next_box_id
            .checked_add(1)
            .ok_or_else(|| RuntimeError {
                message: "box id limit exceeded".to_owned(),
            })?;
        self.boxes.push(BoxOutput {
            id,
            snapshots: vec![BoxSnapshot {
                bar_index: self.bars,
                exists: true,
                left,
                top,
                right,
                bottom,
                bg_color: PineValue::Na,
                border_color: PineValue::Na,
                border_width: PineValue::Int(1),
                border_style: PineValue::String("line.style_solid".to_owned()),
                extend: PineValue::String("extend.none".to_owned()),
                text: PineValue::String(String::new()),
                text_color: PineValue::Na,
            }],
        });
        Ok(PineValue::Box(id))
    }

    pub(super) fn eval_box_set_left(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let left = self.eval_required_box_arg(args, 1, "x")?;
        self.mutate_box(id, |snapshot| {
            snapshot.left = left;
        })
    }

    pub(super) fn eval_box_set_top(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let top = self.eval_required_box_arg(args, 1, "y")?;
        self.mutate_box(id, |snapshot| {
            snapshot.top = top;
        })
    }

    pub(super) fn eval_box_set_right(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let right = self.eval_required_box_arg(args, 1, "x")?;
        self.mutate_box(id, |snapshot| {
            snapshot.right = right;
        })
    }

    pub(super) fn eval_box_set_bottom(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let bottom = self.eval_required_box_arg(args, 1, "y")?;
        self.mutate_box(id, |snapshot| {
            snapshot.bottom = bottom;
        })
    }

    pub(super) fn eval_box_set_lefttop(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let left = self.eval_required_box_arg(args, 1, "x")?;
        let top = self.eval_required_box_arg(args, 2, "y")?;
        self.mutate_box(id, |snapshot| {
            snapshot.left = left;
            snapshot.top = top;
        })
    }

    pub(super) fn eval_box_set_rightbottom(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let right = self.eval_required_box_arg(args, 1, "x")?;
        let bottom = self.eval_required_box_arg(args, 2, "y")?;
        self.mutate_box(id, |snapshot| {
            snapshot.right = right;
            snapshot.bottom = bottom;
        })
    }

    pub(super) fn eval_box_set_bgcolor(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let bg_color = self.eval_required_box_arg(args, 1, "color")?;
        self.mutate_box(id, |snapshot| {
            snapshot.bg_color = bg_color;
        })
    }

    pub(super) fn eval_box_set_border_color(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let border_color = self.eval_required_box_arg(args, 1, "color")?;
        self.mutate_box(id, |snapshot| {
            snapshot.border_color = border_color;
        })
    }

    pub(super) fn eval_box_set_border_width(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let border_width = self.eval_required_box_arg(args, 1, "width")?;
        self.mutate_box(id, |snapshot| {
            snapshot.border_width = border_width;
        })
    }

    pub(super) fn eval_box_set_border_style(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let border_style = self.eval_required_box_arg(args, 1, "style")?;
        self.mutate_box(id, |snapshot| {
            snapshot.border_style = border_style;
        })
    }

    pub(super) fn eval_box_set_extend(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let extend = self.eval_required_box_arg(args, 1, "extend")?;
        self.mutate_box(id, |snapshot| {
            snapshot.extend = extend;
        })
    }

    pub(super) fn eval_box_set_text(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let text = self.eval_required_box_arg(args, 1, "text")?;
        self.mutate_box(id, |snapshot| {
            snapshot.text = text;
        })
    }

    pub(super) fn eval_box_set_text_color(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let text_color = self.eval_required_box_arg(args, 1, "text_color")?;
        self.mutate_box(id, |snapshot| {
            snapshot.text_color = text_color;
        })
    }

    pub(super) fn eval_box_delete(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let Some(id) = id else {
            return Ok(PineValue::Void);
        };
        let Some(box_output) = self.boxes.iter_mut().find(|box_output| box_output.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid box id `{id}`"),
            });
        };
        let Some(latest) = box_output.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("box `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Void);
        }
        let mut next = latest;
        next.bar_index = self.bars;
        next.exists = false;
        box_output.snapshots.push(next);
        Ok(PineValue::Void)
    }

    pub(super) fn eval_box_copy(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_id_arg(args)?;
        let Some(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(box_output) = self.boxes.iter().find(|box_output| box_output.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid box id `{id}`"),
            });
        };
        let Some(latest) = box_output.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("box `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Na);
        }
        if self.boxes.len() >= MAX_BOXES {
            return Err(RuntimeError {
                message: format!("box count cannot exceed {MAX_BOXES}"),
            });
        }
        let copied_id = self.next_box_id;
        self.next_box_id = self
            .next_box_id
            .checked_add(1)
            .ok_or_else(|| RuntimeError {
                message: "box id limit exceeded".to_owned(),
            })?;
        let mut copied = latest;
        copied.bar_index = self.bars;
        self.boxes.push(BoxOutput {
            id: copied_id,
            snapshots: vec![copied],
        });
        Ok(PineValue::Box(copied_id))
    }

    pub(super) fn eval_box_get_top(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_box_get(args, "box.get_top", |snapshot| snapshot.top.clone())
    }

    pub(super) fn eval_box_get_bottom(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_box_get(args, "box.get_bottom", |snapshot| snapshot.bottom.clone())
    }

    pub(super) fn eval_box_get_left(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_box_get(args, "box.get_left", |snapshot| snapshot.left.clone())
    }

    pub(super) fn eval_box_get_right(
        &mut self,
        args: &[HirCallArg],
    ) -> Result<PineValue, RuntimeError> {
        self.eval_box_get(args, "box.get_right", |snapshot| snapshot.right.clone())
    }

    fn eval_box_id_arg(&mut self, args: &[HirCallArg]) -> Result<Option<u32>, RuntimeError> {
        let Some(id_arg) = call_arg_expr(args, 0, "id") else {
            return Err(RuntimeError {
                message: "box mutation missing id argument".to_owned(),
            });
        };
        match self.eval_expr(id_arg)? {
            PineValue::Box(id) => Ok(Some(id)),
            PineValue::Na => Ok(None),
            value => Err(RuntimeError {
                message: format!("box mutation expected box id, got {value:?}"),
            }),
        }
    }

    fn eval_required_box_arg(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
    ) -> Result<PineValue, RuntimeError> {
        let Some(arg) = call_arg_expr(args, index, name) else {
            return Err(RuntimeError {
                message: format!("box call missing {name} argument"),
            });
        };
        self.eval_expr(arg)
    }

    fn mutate_box(
        &mut self,
        id: Option<u32>,
        mutate: impl FnOnce(&mut BoxSnapshot),
    ) -> Result<PineValue, RuntimeError> {
        let Some(id) = id else {
            return Ok(PineValue::Void);
        };
        let Some(box_output) = self.boxes.iter_mut().find(|box_output| box_output.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid box id `{id}`"),
            });
        };
        let Some(latest) = box_output.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("box `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Void);
        }
        let mut next = latest.clone();
        mutate(&mut next);
        if next != latest {
            next.bar_index = self.bars;
            box_output.snapshots.push(next);
        }
        Ok(PineValue::Void)
    }

    fn eval_box_get(
        &mut self,
        args: &[HirCallArg],
        function_name: &str,
        get_value: impl FnOnce(&BoxSnapshot) -> PineValue,
    ) -> Result<PineValue, RuntimeError> {
        let id = self.eval_box_get_id_arg(args, function_name)?;
        let Some(id) = id else {
            return Ok(PineValue::Na);
        };
        let Some(box_output) = self.boxes.iter().find(|box_output| box_output.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid box id `{id}`"),
            });
        };
        let Some(latest) = box_output.snapshots.last() else {
            return Err(RuntimeError {
                message: format!("box `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Na);
        }
        Ok(get_value(latest))
    }

    fn eval_box_get_id_arg(
        &mut self,
        args: &[HirCallArg],
        function_name: &str,
    ) -> Result<Option<u32>, RuntimeError> {
        let Some(id_arg) = call_arg_expr(args, 0, "id") else {
            return Err(RuntimeError {
                message: format!("{function_name} missing id argument"),
            });
        };
        match self.eval_expr(id_arg)? {
            PineValue::Box(id) => Ok(Some(id)),
            PineValue::Na => Ok(None),
            value => Err(RuntimeError {
                message: format!("{function_name} expected box id, got {value:?}"),
            }),
        }
    }
}
