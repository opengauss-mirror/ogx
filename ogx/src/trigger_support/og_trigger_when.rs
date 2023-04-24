use crate::pg_sys;
use crate::trigger_support::{OgTriggerError, TriggerEvent};

/// When a trigger happened
///
/// Maps from a `TEXT` of `BEFORE`, `AFTER`, or `INSTEAD OF`.
///
/// Can be calculated from a `ogx_pg_sys::TriggerEvent`.
pub enum OgTriggerWhen {
    /// `BEFORE`
    Before,
    /// `AFTER`
    After,
    /// `INSTEAD OF`
    InsteadOf,
}

impl TryFrom<TriggerEvent> for OgTriggerWhen {
    type Error = OgTriggerError;
    fn try_from(event: TriggerEvent) -> Result<Self, Self::Error> {
        match event.0 & pg_sys::TRIGGER_EVENT_TIMINGMASK {
            pg_sys::TRIGGER_EVENT_BEFORE => Ok(Self::Before),
            pg_sys::TRIGGER_EVENT_AFTER => Ok(Self::After),
            pg_sys::TRIGGER_EVENT_INSTEAD => Ok(Self::InsteadOf),
            v => Err(OgTriggerError::InvalidOgTriggerWhen(v)),
        }
    }
}

impl ToString for OgTriggerWhen {
    fn to_string(&self) -> String {
        match self {
            OgTriggerWhen::Before => "BEFORE",
            OgTriggerWhen::After => "AFTER",
            OgTriggerWhen::InsteadOf => "INSTEAD OF",
        }
        .to_string()
    }
}
