mod boxes;
mod labels;
mod lines;
mod tables;

use pine_ir::{CallSiteId, HirCallArg};

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
            "label.set_xloc" => self.eval_label_set_xloc(args),
            "label.set_y" => self.eval_label_set_y(args),
            "label.set_xy" => self.eval_label_set_xy(args),
            "label.set_yloc" => self.eval_label_set_yloc(args),
            "label.set_text" => self.eval_label_set_text(args),
            "label.set_color" => self.eval_label_set_color(args),
            "label.set_textcolor" => self.eval_label_set_textcolor(args),
            "label.set_style" => self.eval_label_set_style(args),
            "label.set_size" => self.eval_label_set_size(args),
            "label.set_tooltip" => self.eval_label_set_tooltip(args),
            "label.delete" => self.eval_label_delete(args),
            "label.copy" => self.eval_label_copy(args),
            "label.get_x" => self.eval_label_get_x(args),
            "label.get_y" => self.eval_label_get_y(args),
            "label.get_text" => self.eval_label_get_text(args),
            "line.new" => self.eval_line_new(args),
            "line.set_x1" => self.eval_line_set_x1(args),
            "line.set_y1" => self.eval_line_set_y1(args),
            "line.set_xy1" => self.eval_line_set_xy1(args),
            "line.set_x2" => self.eval_line_set_x2(args),
            "line.set_y2" => self.eval_line_set_y2(args),
            "line.set_xy2" => self.eval_line_set_xy2(args),
            "line.set_color" => self.eval_line_set_color(args),
            "line.set_width" => self.eval_line_set_width(args),
            "line.set_style" => self.eval_line_set_style(args),
            "line.set_extend" => self.eval_line_set_extend(args),
            "line.delete" => self.eval_line_delete(args),
            "line.copy" => self.eval_line_copy(args),
            "box.new" => self.eval_box_new(args),
            "box.set_left" => self.eval_box_set_left(args),
            "box.set_top" => self.eval_box_set_top(args),
            "box.set_right" => self.eval_box_set_right(args),
            "box.set_bottom" => self.eval_box_set_bottom(args),
            "box.set_lefttop" => self.eval_box_set_lefttop(args),
            "box.set_rightbottom" => self.eval_box_set_rightbottom(args),
            "box.set_bgcolor" => self.eval_box_set_bgcolor(args),
            "box.set_border_color" => self.eval_box_set_border_color(args),
            "box.set_border_width" => self.eval_box_set_border_width(args),
            "box.set_border_style" => self.eval_box_set_border_style(args),
            "box.delete" => self.eval_box_delete(args),
            "box.copy" => self.eval_box_copy(args),
            "table.new" => self.eval_table_new(args),
            "table.cell" => self.eval_table_cell(args),
            _ => return None,
        })
    }
}
