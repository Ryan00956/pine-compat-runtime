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
