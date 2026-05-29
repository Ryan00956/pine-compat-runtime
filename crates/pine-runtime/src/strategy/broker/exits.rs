#[derive(Debug, Clone, PartialEq)]
pub(super) enum PendingExitTrigger {
    Stop(f64),
    Limit(f64),
}

impl PendingExitTrigger {
    pub(super) fn price(&self) -> f64 {
        match self {
            Self::Stop(price) | Self::Limit(price) => *price,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PendingExit {
    pub(super) id: String,
    pub(super) from_entry: String,
    pub(super) trigger: PendingExitTrigger,
    pub(super) last_update_bar_index: usize,
}
