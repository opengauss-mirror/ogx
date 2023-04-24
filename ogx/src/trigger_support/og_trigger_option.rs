use crate::pg_sys;
use crate::trigger_support::{OgTriggerError, TriggerEvent};

/// The operation for which the trigger was fired
///
/// Maps from a `TEXT` of `INSERT`, `UPDATE`, `DELETE`, or `TRUNCATE`.
///
/// Can be calculated from a `ogx_pg_sys::TriggerEvent`.
// Postgres constants: https://cs.github.com/postgres/postgres/blob/36d4efe779bfc7190ea1c1cf8deb0d945b726663/src/include/commands/trigger.h#L92
// Postgres defines: https://cs.github.com/postgres/postgres/blob/36d4efe779bfc7190ea1c1cf8deb0d945b726663/src/include/commands/trigger.h#L92
pub enum OgTriggerOperation {
    /// `INSERT`
    Insert,
    /// `UPDATE`
    Update,
    /// `DELETE`
    Delete,
    /// `TRUNCATE`
    Truncate,
}

impl TryFrom<TriggerEvent> for OgTriggerOperation {
    type Error = OgTriggerError;
    fn try_from(event: TriggerEvent) -> Result<Self, Self::Error> {
        match event.0 & pg_sys::TRIGGER_EVENT_OPMASK {
            pg_sys::TRIGGER_EVENT_INSERT => Ok(Self::Insert),
            pg_sys::TRIGGER_EVENT_DELETE => Ok(Self::Delete),
            pg_sys::TRIGGER_EVENT_UPDATE => Ok(Self::Update),
            pg_sys::TRIGGER_EVENT_TRUNCATE => Ok(Self::Truncate),
            v => Err(OgTriggerError::InvalidOgTriggerOperation(v)),
        }
    }
}

impl ToString for OgTriggerOperation {
    fn to_string(&self) -> String {
        match self {
            OgTriggerOperation::Insert => "INSERT",
            OgTriggerOperation::Update => "UPDATE",
            OgTriggerOperation::Delete => "DELETE",
            OgTriggerOperation::Truncate => "TRUNCATE",
        }
        .to_string()
    }
}
