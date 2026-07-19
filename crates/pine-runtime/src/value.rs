#[derive(Debug, Clone, PartialEq)]
pub struct ChartPointValue {
    pub time: Box<PineValue>,
    pub index: Box<PineValue>,
    pub price: Box<PineValue>,
}

impl ChartPointValue {
    #[must_use]
    pub fn new(time: PineValue, index: PineValue, price: PineValue) -> Self {
        Self {
            time: Box::new(time),
            index: Box::new(index),
            price: Box::new(price),
        }
    }

    #[must_use]
    pub fn field(&self, index: usize) -> PineValue {
        match index {
            0 => (*self.time).clone(),
            1 => (*self.index).clone(),
            2 => (*self.price).clone(),
            _ => PineValue::Na,
        }
    }

    pub fn set_field(&mut self, index: usize, value: PineValue) {
        match index {
            0 => *self.time = value,
            1 => *self.index = value,
            2 => *self.price = value,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PineValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(u64),
    Plot(u32),
    HLine(u32),
    Label(u32),
    Line(u32),
    LineFill(u32),
    Polyline(u32),
    Box(u32),
    Table(u32),
    ChartPoint(ChartPointValue),
    Array(u32),
    Matrix(u32),
    Map(u32),
    UserType(Vec<PineValue>),
    Tuple(Vec<PineValue>),
    Na,
    Void,
}

/// Encodes an RGB or RGBA literal without conflating low-valued RGBA payloads
/// (for example transparent green) with ordinary `0xRRGGBB` colors.
#[must_use]
pub fn encode_color_literal(value: u32, includes_alpha: bool) -> u64 {
    if !includes_alpha {
        return u64::from(value);
    }
    encode_color_rgba(value >> 8, value & 0xFF)
}

/// Encodes separate RGB and alpha channels into Pine's unambiguous color value.
#[must_use]
pub fn encode_color_rgba(rgb: u32, alpha: u32) -> u64 {
    let rgb = rgb & 0xFF_FFFF;
    let alpha = alpha & 0xFF;
    if alpha == 0xFF {
        return u64::from(rgb);
    }
    let encoded = u64::from((rgb << 8) | alpha);
    if encoded <= 0xFF_FFFF {
        (1 << 32) | encoded
    } else {
        encoded
    }
}

impl PineValue {
    #[must_use]
    pub fn is_na(&self) -> bool {
        matches!(self, Self::Na)
    }

    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }
}
