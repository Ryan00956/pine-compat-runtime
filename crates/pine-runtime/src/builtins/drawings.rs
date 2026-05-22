use pine_ir::{CallSiteId, HirCallArg};

use crate::builtins::args::call_arg_expr;
use crate::*;

impl<'a> HistoricalRuntime<'a> {
    pub(crate) fn eval_drawing_call(
        &mut self,
        callee: &str,
        _call_site_id: CallSiteId,
        args: &[HirCallArg],
    ) -> Option<Result<PineValue, RuntimeError>> {
        Some(match callee {
            "label.new" => self.eval_label_new(args),
            "label.set_x" => self.eval_label_set_x(args),
            "label.set_y" => self.eval_label_set_y(args),
            "label.set_xy" => self.eval_label_set_xy(args),
            "label.set_text" => self.eval_label_set_text(args),
            "label.set_color" => self.eval_label_set_color(args),
            "label.set_textcolor" => self.eval_label_set_textcolor(args),
            "label.set_style" => self.eval_label_set_style(args),
            "label.set_size" => self.eval_label_set_size(args),
            "label.set_tooltip" => self.eval_label_set_tooltip(args),
            "label.delete" => self.eval_label_delete(args),
            "line.new" => self.eval_line_new(args),
            _ => return None,
        })
    }

    fn eval_label_new(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let Some(x_arg) = call_arg_expr(args, 0, "x") else {
            return Err(RuntimeError {
                message: "label.new missing x argument".to_owned(),
            });
        };
        let Some(y_arg) = call_arg_expr(args, 1, "y") else {
            return Err(RuntimeError {
                message: "label.new missing y argument".to_owned(),
            });
        };
        let Some(text_arg) = call_arg_expr(args, 2, "text") else {
            return Err(RuntimeError {
                message: "label.new missing text argument".to_owned(),
            });
        };

        let x = self.eval_expr(x_arg)?;
        let y = self.eval_expr(y_arg)?;
        let text = self.eval_expr(text_arg)?;
        let xloc = self.eval_label_option(args, 3, "xloc", "xloc.bar_index")?;
        let yloc = self.eval_label_option(args, 4, "yloc", "yloc.price")?;
        let color = self.eval_label_option_value(args, 5, "color", PineValue::Na)?;
        let style = self.eval_label_option(args, 6, "style", "label.style_label_down")?;
        let text_color = self.eval_label_option_value(args, 7, "textcolor", PineValue::Na)?;
        let size = self.eval_label_option(args, 8, "size", "size.normal")?;
        let tooltip =
            self.eval_label_option_value(args, 9, "tooltip", PineValue::String(String::new()))?;
        if self.labels.len() >= MAX_LABELS {
            return Err(RuntimeError {
                message: format!("label count cannot exceed {MAX_LABELS}"),
            });
        }
        let id = self.next_label_id;
        self.next_label_id = self
            .next_label_id
            .checked_add(1)
            .ok_or_else(|| RuntimeError {
                message: "label id limit exceeded".to_owned(),
            })?;
        self.labels.push(LabelOutput {
            id,
            snapshots: vec![LabelSnapshot {
                bar_index: self.bars,
                exists: true,
                x,
                y,
                text,
                xloc,
                yloc,
                color,
                style,
                text_color,
                size,
                tooltip,
            }],
        });
        Ok(PineValue::Label(id))
    }

    fn eval_line_new(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let x1 = self.eval_required_line_arg(args, 0, "x1")?;
        let y1 = self.eval_required_line_arg(args, 1, "y1")?;
        let x2 = self.eval_required_line_arg(args, 2, "x2")?;
        let y2 = self.eval_required_line_arg(args, 3, "y2")?;
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
            }],
        });
        Ok(PineValue::Line(id))
    }

    fn eval_label_set_x(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let x = self.eval_required_label_arg(args, 1, "x")?;
        self.mutate_label(id, |snapshot| {
            snapshot.x = x;
        })
    }

    fn eval_label_set_y(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let y = self.eval_required_label_arg(args, 1, "y")?;
        self.mutate_label(id, |snapshot| {
            snapshot.y = y;
        })
    }

    fn eval_label_set_xy(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let x = self.eval_required_label_arg(args, 1, "x")?;
        let y = self.eval_required_label_arg(args, 2, "y")?;
        self.mutate_label(id, |snapshot| {
            snapshot.x = x;
            snapshot.y = y;
        })
    }

    fn eval_label_set_text(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let text = self.eval_required_label_arg(args, 1, "text")?;
        self.mutate_label(id, |snapshot| {
            snapshot.text = text;
        })
    }

    fn eval_label_set_color(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let color = self.eval_required_label_arg(args, 1, "color")?;
        self.mutate_label(id, |snapshot| {
            snapshot.color = color;
        })
    }

    fn eval_label_set_textcolor(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let text_color = self.eval_required_label_arg(args, 1, "textcolor")?;
        self.mutate_label(id, |snapshot| {
            snapshot.text_color = text_color;
        })
    }

    fn eval_label_set_style(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let style = self.eval_required_label_arg(args, 1, "style")?;
        self.mutate_label(id, |snapshot| {
            snapshot.style = style;
        })
    }

    fn eval_label_set_size(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let size = self.eval_required_label_arg(args, 1, "size")?;
        self.mutate_label(id, |snapshot| {
            snapshot.size = size;
        })
    }

    fn eval_label_set_tooltip(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let tooltip = self.eval_required_label_arg(args, 1, "tooltip")?;
        self.mutate_label(id, |snapshot| {
            snapshot.tooltip = tooltip;
        })
    }

    fn eval_label_delete(&mut self, args: &[HirCallArg]) -> Result<PineValue, RuntimeError> {
        let id = self.eval_label_id_arg(args)?;
        let Some(id) = id else {
            return Ok(PineValue::Void);
        };
        let Some(label) = self.labels.iter_mut().find(|label| label.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid label id `{id}`"),
            });
        };
        let Some(latest) = label.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("label `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Void);
        }
        let mut next = latest;
        next.bar_index = self.bars;
        next.exists = false;
        label.snapshots.push(next);
        Ok(PineValue::Void)
    }

    fn eval_label_id_arg(&mut self, args: &[HirCallArg]) -> Result<Option<u32>, RuntimeError> {
        let Some(id_arg) = call_arg_expr(args, 0, "id") else {
            return Err(RuntimeError {
                message: "label mutation missing id argument".to_owned(),
            });
        };
        match self.eval_expr(id_arg)? {
            PineValue::Label(id) => Ok(Some(id)),
            PineValue::Na => Ok(None),
            value => Err(RuntimeError {
                message: format!("label mutation expected label id, got {value:?}"),
            }),
        }
    }

    fn eval_required_label_arg(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
    ) -> Result<PineValue, RuntimeError> {
        let Some(arg) = call_arg_expr(args, index, name) else {
            return Err(RuntimeError {
                message: format!("label mutation missing {name} argument"),
            });
        };
        self.eval_expr(arg)
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

    fn mutate_label(
        &mut self,
        id: Option<u32>,
        mutate: impl FnOnce(&mut LabelSnapshot),
    ) -> Result<PineValue, RuntimeError> {
        let Some(id) = id else {
            return Ok(PineValue::Void);
        };
        let Some(label) = self.labels.iter_mut().find(|label| label.id == id) else {
            return Err(RuntimeError {
                message: format!("invalid label id `{id}`"),
            });
        };
        let Some(latest) = label.snapshots.last().cloned() else {
            return Err(RuntimeError {
                message: format!("label `{id}` has no snapshots"),
            });
        };
        if !latest.exists {
            return Ok(PineValue::Void);
        }
        let mut next = latest.clone();
        mutate(&mut next);
        if next != latest {
            next.bar_index = self.bars;
            label.snapshots.push(next);
        }
        Ok(PineValue::Void)
    }

    fn eval_label_option(
        &mut self,
        args: &[HirCallArg],
        index: usize,
        name: &str,
        default: &str,
    ) -> Result<PineValue, RuntimeError> {
        self.eval_label_option_value(args, index, name, PineValue::String(default.to_owned()))
    }

    fn eval_label_option_value(
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
}
