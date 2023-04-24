/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
/*!

Rust to SQL mapping support.

> Like all of the [`sql_entity_graph`][crate::sql_entity_graph] APIs, this is considered **internal**
to the `ogx` framework and very subject to change between versions. While you may use this, please do it with caution.

*/
pub(crate) mod aggregate;
pub(crate) mod control_file;
pub(crate) mod extension_sql;
pub(crate) mod mapping;
pub mod metadata;
pub(crate) mod og_extern;
pub(crate) mod og_trigger;
pub(crate) mod ogx_attribute;
pub(crate) mod ogx_sql;
pub(crate) mod positioning_ref;
pub(crate) mod og_enum;
pub(crate) mod og_hash;
pub(crate) mod og_ord;
pub(crate) mod og_type;
pub(crate) mod schema;
pub(crate) mod to_sql;
pub(crate) mod used_type;

pub use aggregate::entity::{AggregateTypeEntity, OgAggregateEntity};
pub use aggregate::{
    AggregateType, AggregateTypeList, FinalizeModify, ParallelOption, OgAggregate,
};
pub use control_file::ControlFile;
pub use extension_sql::entity::{ExtensionSqlEntity, SqlDeclaredEntity};
pub use extension_sql::{ExtensionSql, ExtensionSqlFile, SqlDeclared};
pub use mapping::{RustSourceOnlySqlMapping, RustSqlMapping};
pub use og_extern::entity::{
    OgExternArgumentEntity, OgExternEntity, OgExternReturnEntity, OgExternReturnEntityIteratedItem,
    OgOperatorEntity,
};
pub use og_extern::{NameMacro, OgExtern, OgExternArgument, OgOperator};
pub use og_trigger::attribute::OgTriggerAttribute;
pub use og_trigger::entity::OgTriggerEntity;
pub use og_trigger::OgTrigger;
pub use ogx_sql::{OgxSql, RustToSqlMapping};
pub use positioning_ref::PositioningRef;
pub use og_enum::entity::OgEnumEntity;
pub use og_enum::OgEnum;
pub use og_hash::entity::OgHashEntity;
pub use og_hash::OgHash;
pub use og_ord::entity::OgOrdEntity;
pub use og_ord::OgOrd;
pub use og_type::entity::OgTypeEntity;
pub use og_type::OgType;
pub use schema::entity::SchemaEntity;
pub use schema::Schema;
pub use to_sql::entity::ToSqlConfigEntity;
pub use to_sql::{ToSql, ToSqlConfig};
pub use used_type::{UsedType, UsedTypeEntity};

pub use crate::ExternArgs;

/// Able to produce a GraphViz DOT format identifier.
pub trait SqlGraphIdentifier {
    /// A dot style identifier for the entity.
    ///
    /// Typically this is a 'archetype' prefix (eg `fn` or `type`) then result of
    /// [`std::module_path`], [`core::any::type_name`], or some combination of [`std::file`] and
    /// [`std::line`].
    fn dot_identifier(&self) -> String;

    /// A Rust identifier for the entity.
    ///
    /// Typically this is the result of [`std::module_path`], [`core::any::type_name`],
    /// or some combination of [`std::file`] and [`std::line`].
    fn rust_identifier(&self) -> String;

    fn file(&self) -> Option<&'static str>;

    fn line(&self) -> Option<u32>;
}

/// An entity corresponding to some SQL required by the extension.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SqlGraphEntity {
    ExtensionRoot(ControlFile),
    Schema(SchemaEntity),
    CustomSql(ExtensionSqlEntity),
    Function(OgExternEntity),
    Type(OgTypeEntity),
    BuiltinType(String),
    Enum(OgEnumEntity),
    Ord(OgOrdEntity),
    Hash(OgHashEntity),
    Aggregate(OgAggregateEntity),
    Trigger(OgTriggerEntity),
}

impl SqlGraphEntity {
    pub fn sql_anchor_comment(&self) -> String {
        let maybe_file_and_line = if let (Some(file), Some(line)) = (self.file(), self.line()) {
            format!("-- {file}:{line}\n", file = file, line = line)
        } else {
            String::default()
        };
        format!(
            "\
            {maybe_file_and_line}\
            -- {rust_identifier}\
        ",
            maybe_file_and_line = maybe_file_and_line,
            rust_identifier = self.rust_identifier(),
        )
    }
}

impl SqlGraphIdentifier for SqlGraphEntity {
    fn dot_identifier(&self) -> String {
        match self {
            SqlGraphEntity::Schema(item) => item.dot_identifier(),
            SqlGraphEntity::CustomSql(item) => item.dot_identifier(),
            SqlGraphEntity::Function(item) => item.dot_identifier(),
            SqlGraphEntity::Type(item) => item.dot_identifier(),
            SqlGraphEntity::BuiltinType(item) => format!("preexisting type {}", item),
            SqlGraphEntity::Enum(item) => item.dot_identifier(),
            SqlGraphEntity::Ord(item) => item.dot_identifier(),
            SqlGraphEntity::Hash(item) => item.dot_identifier(),
            SqlGraphEntity::Aggregate(item) => item.dot_identifier(),
            SqlGraphEntity::Trigger(item) => item.dot_identifier(),
            SqlGraphEntity::ExtensionRoot(item) => item.dot_identifier(),
        }
    }

    fn rust_identifier(&self) -> String {
        match self {
            SqlGraphEntity::Schema(item) => item.rust_identifier(),
            SqlGraphEntity::CustomSql(item) => item.rust_identifier(),
            SqlGraphEntity::Function(item) => item.rust_identifier(),
            SqlGraphEntity::Type(item) => item.rust_identifier(),
            SqlGraphEntity::BuiltinType(item) => item.to_string(),
            SqlGraphEntity::Enum(item) => item.rust_identifier(),
            SqlGraphEntity::Ord(item) => item.rust_identifier(),
            SqlGraphEntity::Hash(item) => item.rust_identifier(),
            SqlGraphEntity::Aggregate(item) => item.rust_identifier(),
            SqlGraphEntity::Trigger(item) => item.rust_identifier(),
            SqlGraphEntity::ExtensionRoot(item) => item.rust_identifier(),
        }
    }

    fn file(&self) -> Option<&'static str> {
        match self {
            SqlGraphEntity::Schema(item) => item.file(),
            SqlGraphEntity::CustomSql(item) => item.file(),
            SqlGraphEntity::Function(item) => item.file(),
            SqlGraphEntity::Type(item) => item.file(),
            SqlGraphEntity::BuiltinType(_item) => None,
            SqlGraphEntity::Enum(item) => item.file(),
            SqlGraphEntity::Ord(item) => item.file(),
            SqlGraphEntity::Hash(item) => item.file(),
            SqlGraphEntity::Aggregate(item) => item.file(),
            SqlGraphEntity::Trigger(item) => item.file(),
            SqlGraphEntity::ExtensionRoot(item) => item.file(),
        }
    }

    fn line(&self) -> Option<u32> {
        match self {
            SqlGraphEntity::Schema(item) => item.line(),
            SqlGraphEntity::CustomSql(item) => item.line(),
            SqlGraphEntity::Function(item) => item.line(),
            SqlGraphEntity::Type(item) => item.line(),
            SqlGraphEntity::BuiltinType(_item) => None,
            SqlGraphEntity::Enum(item) => item.line(),
            SqlGraphEntity::Ord(item) => item.line(),
            SqlGraphEntity::Hash(item) => item.line(),
            SqlGraphEntity::Aggregate(item) => item.line(),
            SqlGraphEntity::Trigger(item) => item.line(),
            SqlGraphEntity::ExtensionRoot(item) => item.line(),
        }
    }
}

impl ToSql for SqlGraphEntity {
    #[tracing::instrument(level = "debug", skip(self, context), fields(identifier = %self.rust_identifier()))]
    fn to_sql(&self, context: &OgxSql) -> eyre::Result<String> {
        match self {
            SqlGraphEntity::Schema(item) => {
                if item.name != "public" && item.name != "pg_catalog" {
                    item.to_sql(context)
                } else {
                    Ok(String::default())
                }
            }
            SqlGraphEntity::CustomSql(item) => item.to_sql(context),
            SqlGraphEntity::Function(item) => {
                if let Some(result) = item.to_sql_config.to_sql(self, context) {
                    return result;
                }
                if context.graph.neighbors_undirected(context.externs.get(item).unwrap().clone()).any(|neighbor| {
                    let neighbor_item = &context.graph[neighbor];
                    match neighbor_item {
                        SqlGraphEntity::Type(OgTypeEntity { in_fn, in_fn_module_path, out_fn, out_fn_module_path, .. }) => {
                            let is_in_fn = item.full_path.starts_with(in_fn_module_path) && item.full_path.ends_with(in_fn);
                            if is_in_fn {
                                tracing::trace!(r#type = %neighbor_item.dot_identifier(), "Skipping, is an in_fn.");
                            }
                            let is_out_fn = item.full_path.starts_with(out_fn_module_path) && item.full_path.ends_with(out_fn);
                            if is_out_fn {
                                tracing::trace!(r#type = %neighbor_item.dot_identifier(), "Skipping, is an out_fn.");
                            }
                            is_in_fn || is_out_fn
                        },
                        _ => false,
                    }
                }) {
                    Ok(String::default())
                } else {
                    item.to_sql(context)
                }
            }
            SqlGraphEntity::Type(item) => {
                item.to_sql_config.to_sql(self, context).unwrap_or_else(|| item.to_sql(context))
            }
            SqlGraphEntity::BuiltinType(_) => Ok(String::default()),
            SqlGraphEntity::Enum(item) => {
                item.to_sql_config.to_sql(self, context).unwrap_or_else(|| item.to_sql(context))
            }
            SqlGraphEntity::Ord(item) => {
                item.to_sql_config.to_sql(self, context).unwrap_or_else(|| item.to_sql(context))
            }
            SqlGraphEntity::Hash(item) => {
                item.to_sql_config.to_sql(self, context).unwrap_or_else(|| item.to_sql(context))
            }
            SqlGraphEntity::Aggregate(item) => {
                item.to_sql_config.to_sql(self, context).unwrap_or_else(|| item.to_sql(context))
            }
            SqlGraphEntity::Trigger(item) => {
                item.to_sql_config.to_sql(self, context).unwrap_or_else(|| item.to_sql(context))
            }
            SqlGraphEntity::ExtensionRoot(item) => item.to_sql(context),
        }
    }
}
