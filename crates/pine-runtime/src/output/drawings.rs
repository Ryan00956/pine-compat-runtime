use crate::PineValue;

#[derive(Debug, Clone, PartialEq)]
pub struct LabelOutput {
    pub id: u32,
    pub snapshots: Vec<LabelSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSnapshot {
    pub bar_index: usize,
    pub exists: bool,
    pub x: PineValue,
    pub y: PineValue,
    pub text: PineValue,
    pub xloc: PineValue,
    pub yloc: PineValue,
    pub color: PineValue,
    pub style: PineValue,
    pub text_color: PineValue,
    pub size: PineValue,
    pub tooltip: PineValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineOutput {
    pub id: u32,
    pub snapshots: Vec<LineSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineSnapshot {
    pub bar_index: usize,
    pub exists: bool,
    pub x1: PineValue,
    pub y1: PineValue,
    pub x2: PineValue,
    pub y2: PineValue,
    pub color: PineValue,
    pub width: PineValue,
    pub style: PineValue,
    pub extend: PineValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxOutput {
    pub id: u32,
    pub snapshots: Vec<BoxSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxSnapshot {
    pub bar_index: usize,
    pub exists: bool,
    pub left: PineValue,
    pub top: PineValue,
    pub right: PineValue,
    pub bottom: PineValue,
    pub bg_color: PineValue,
    pub border_color: PineValue,
    pub border_width: PineValue,
    pub border_style: PineValue,
}
