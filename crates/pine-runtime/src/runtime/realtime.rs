use pine_ir::HirProgram;

use crate::*;

pub struct RealtimeRuntime<'a> {
    confirmed: HistoricalRuntime<'a>,
    forming: Option<HistoricalRuntime<'a>>,
}
impl<'a> RealtimeRuntime<'a> {
    #[must_use]
    pub fn new(program: &'a HirProgram) -> Self {
        Self::with_request_environment(program, RequestEnvironment::default())
    }

    #[must_use]
    pub fn with_request_environment(
        program: &'a HirProgram,
        request_environment: RequestEnvironment,
    ) -> Self {
        Self {
            confirmed: HistoricalRuntime::with_request_environment(program, request_environment),
            forming: None,
        }
    }

    #[must_use]
    pub fn with_request_environment_and_input_overrides(
        program: &'a HirProgram,
        request_environment: RequestEnvironment,
        input_overrides: InputOverrides,
    ) -> Self {
        Self {
            confirmed: HistoricalRuntime::with_request_environment_and_input_overrides(
                program,
                request_environment,
                input_overrides,
            ),
            forming: None,
        }
    }

    #[must_use]
    pub fn request_environment(&self) -> &RequestEnvironment {
        self.forming
            .as_ref()
            .unwrap_or(&self.confirmed)
            .request_environment()
    }

    pub fn update(&mut self, update: BarUpdate) -> Result<RuntimeResult, RuntimeError> {
        self.update_inner(update, None)
    }

    pub fn update_with_execution_time(
        &mut self,
        update: BarUpdate,
        execution_time: i64,
    ) -> Result<RuntimeResult, RuntimeError> {
        self.update_inner(update, Some(execution_time))
    }

    fn update_inner(
        &mut self,
        update: BarUpdate,
        execution_time: Option<i64>,
    ) -> Result<RuntimeResult, RuntimeError> {
        match update.kind {
            BarUpdateKind::Historical => {
                let mut runtime = self.confirmed.clone();
                runtime.append_bar_with_context(update.bar, update.kind, true, execution_time)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Confirmed => {
                let runtime = self.replay_from_confirmed(update, execution_time)?;
                self.confirmed = runtime;
                self.forming = None;
                Ok(self.confirmed.result())
            }
            BarUpdateKind::Forming => {
                if !self.executes_strategy_on_forming() {
                    return Ok(self.confirmed.result());
                }
                let runtime = self.replay_from_confirmed(update, execution_time)?;
                let result = runtime.result();
                self.forming = Some(runtime);
                Ok(result)
            }
        }
    }

    fn executes_strategy_on_forming(&self) -> bool {
        self.confirmed.program.script_mode != pine_ir::ScriptMode::Strategy
            || self.confirmed.program.strategy_settings.calc_on_every_tick
    }

    fn replay_from_confirmed(
        &mut self,
        update: BarUpdate,
        execution_time: Option<i64>,
    ) -> Result<HistoricalRuntime<'a>, RuntimeError> {
        let is_new_bar = self.forming.is_none();
        let mut runtime = self.confirmed.clone();
        if let Some(previous_forming) = &self.forming {
            runtime.seed_intrabar_persistence_from(previous_forming);
        }
        runtime.restore_strategy_checkpoint(&self.confirmed);
        runtime.append_bar_with_context(update.bar, update.kind, is_new_bar, execution_time)?;
        Ok(runtime)
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
