use crate::heap_tuple::PgHeapTuple;
use crate::pg_sys;
use crate::ogbox::AllocatedByOpenGauss;
use crate::trigger_support::{OgTriggerLevel, OgTriggerOperation, OgTriggerWhen, TriggerEvent};

pub struct OgTriggerSafe<'a> {
    pub name: &'a str,
    pub new: Option<PgHeapTuple<'a, AllocatedByOpenGauss>>,
    pub current: Option<PgHeapTuple<'a, AllocatedByOpenGauss>>,
    pub event: TriggerEvent,
    pub when: OgTriggerWhen,
    pub level: OgTriggerLevel,
    pub op: OgTriggerOperation,
    pub relid: pg_sys::Oid,
    pub old_transition_table_name: Option<&'a str>,
    pub new_transition_table_name: Option<&'a str>,
    pub relation: crate::PgRelation,
    pub table_name: String,
    pub table_schema: String,
    pub extra_args: Vec<String>,
}
