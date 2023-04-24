#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum OgTriggerError {
    #[error("`OgTrigger`s can only be built from `FunctionCallInfo` instances which `ogx::pg_sys::called_as_trigger(fcinfo)` returns `true`")]
    NotTrigger,
    #[error("`OgTrigger`s cannot be built from `NULL` `ogx::pg_sys::FunctionCallInfo`s")]
    NullFunctionCallInfo,
    #[error(
        "`InvalidOgTriggerWhen` cannot be built from `event & TRIGGER_EVENT_TIMINGMASK` of `{0}"
    )]
    InvalidOgTriggerWhen(u32),
    #[error(
        "`InvalidOgTriggerOperation` cannot be built from `event & TRIGGER_EVENT_OPMASK` of `{0}"
    )]
    InvalidOgTriggerOperation(u32),
    #[error("core::str::Utf8Error: {0}")]
    CoreUtf8(#[from] core::str::Utf8Error),
    #[error("TryFromIntError: {0}")]
    TryFromInt(#[from] core::num::TryFromIntError),
    #[error("The `ogx::pg_sys::TriggerData`'s `tg_trigger` field was a NULL pointer")]
    NullTrigger,
    #[error("The `ogx::pg_sys::FunctionCallInfo`'s `context` field was a NULL pointer")]
    NullTriggerData,
    #[error("The `ogx::pg_sys::TriggerData`'s `tg_relation` field was a NULL pointer")]
    NullRelation,
}
