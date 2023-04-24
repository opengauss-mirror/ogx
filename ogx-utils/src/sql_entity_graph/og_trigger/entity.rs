/*!

`#[og_trigger]` related entities for Rust to SQL translation

> Like all of the [`sql_entity_graph`][crate::sql_entity_graph] APIs, this is considered **internal**
to the `ogx` framework and very subject to change between versions. While you may use this, please do it with caution.

*/
use crate::sql_entity_graph::{
    OgxSql, SqlGraphEntity, SqlGraphIdentifier, ToSql, ToSqlConfigEntity,
};
use core::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use core::fmt::Debug;
use core::hash::Hash;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OgTriggerEntity {
    pub function_name: &'static str,
    pub to_sql_config: ToSqlConfigEntity,
    pub file: &'static str,
    pub line: u32,
    pub module_path: &'static str,
    pub full_path: &'static str,
}

impl OgTriggerEntity {
    fn wrapper_function_name(&self) -> String {
        self.function_name.to_string() + "_wrapper"
    }
}

impl Ord for OgTriggerEntity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.full_path.cmp(other.full_path)
    }
}

impl PartialOrd for OgTriggerEntity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<OgTriggerEntity> for SqlGraphEntity {
    fn from(val: OgTriggerEntity) -> Self {
        SqlGraphEntity::Trigger(val)
    }
}

impl ToSql for OgTriggerEntity {
    #[tracing::instrument(
        level = "error",
        skip(self, context),
        fields(identifier = %self.rust_identifier()),
    )]
    fn to_sql(&self, context: &OgxSql) -> eyre::Result<String> {
        let self_index = context.triggers[self];
        let schema = context.schema_prefix_for(&self_index);

        let sql = format!(
            "\n\
            -- {file}:{line}\n\
            -- {full_path}\n\
            CREATE FUNCTION {schema}\"{function_name}\"()\n\
                \tRETURNS TRIGGER\n\
                \tLANGUAGE c\n\
                \tAS 'MODULE_PATHNAME', '{wrapper_function_name}';\
        ",
            schema = schema,
            file = self.file,
            line = self.line,
            full_path = self.full_path,
            function_name = self.function_name,
            wrapper_function_name = self.wrapper_function_name(),
        );
        Ok(sql)
    }
}

impl SqlGraphIdentifier for OgTriggerEntity {
    fn dot_identifier(&self) -> String {
        format!("trigger fn {}", self.full_path)
    }
    fn rust_identifier(&self) -> String {
        self.full_path.to_string()
    }

    fn file(&self) -> Option<&'static str> {
        Some(self.file)
    }

    fn line(&self) -> Option<u32> {
        Some(self.line)
    }
}
