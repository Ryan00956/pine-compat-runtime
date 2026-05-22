use pine_ir::HirProgram;

use crate::*;

pub struct RealtimeRuntime<'a> {
    confirmed: HistoricalRuntime<'a>,
    forming: Option<HistoricalRuntime<'a>>,
}
impl<'a> RealtimeRuntime<'a> {
    #[must_use]
    pub fn new(program: &'a HirProgram) -> Self {
        Self {
            confirmed: HistoricalRuntime::new(program),
            forming: None,
        }
    }

    pub fn update(&mut self, update: BarUpdate) -> Result<RuntimeResult, RuntimeError> {
        match update.kind {
            BarUpdateKind::Historical => {
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_kind(update.bar, update.kind)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Confirmed => {
                let is_new_bar = self.forming.is_none();
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_context(update.bar, update.kind, is_new_bar)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Forming => {
                let is_new_bar = self.forming.is_none();
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_context(update.bar, update.kind, is_new_bar)?;
                let result = runtime.result();
                self.forming = Some(runtime);
                Ok(result)
            }
        }
    }

    #[must_use]
    pub fn result(&self) -> RuntimeResult {
        self.forming.as_ref().unwrap_or(&self.confirmed).result()
    }

    #[must_use]
    pub fn confirmed_result(&self) -> RuntimeResult {
        self.confirmed.result()
    }

    #[must_use]
    pub fn profile(&self) -> RuntimeProfile {
        self.forming.as_ref().unwrap_or(&self.confirmed).profile()
    }

    #[must_use]
    pub fn confirmed_profile(&self) -> RuntimeProfile {
        self.confirmed.profile()
    }
}
