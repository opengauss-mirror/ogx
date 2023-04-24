/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
/*!

`ogx_module_magic!()` related macro expansion for Rust to SQL translation

> Like all of the [`sql_entity_graph`][crate::sql_entity_graph] APIs, this is considered **internal**
to the `ogx` framework and very subject to change between versions. While you may use this, please do it with caution.

*/
use super::{SqlGraphEntity, SqlGraphIdentifier, ToSql};
use core::convert::TryFrom;
use std::collections::HashMap;
use tracing_error::SpanTrace;

/// The parsed contents of a `.control` file.
///
/// ```rust
/// use ogx_utils::sql_entity_graph::ControlFile;
/// use std::convert::TryFrom;
/// # fn main() -> eyre::Result<()> {
/// let context = include_str!("../../../ogx-examples/custom_types/custom_types.control");
/// let _control_file = ControlFile::try_from(context)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ControlFile {
    pub comment: String,
    pub default_version: String,
    pub module_pathname: Option<String>,
    pub relocatable: bool,
    pub superuser: bool,
    pub schema: Option<String>,
}

impl ControlFile {
    /// Parse a `.control` file.
    ///
    /// ```rust
    /// use ogx_utils::sql_entity_graph::ControlFile;
    /// # fn main() -> eyre::Result<()> {
    /// let context = include_str!("../../../ogx-examples/custom_types/custom_types.control");
    /// let _control_file = ControlFile::from_str(context)?;
    /// # Ok(())
    /// # }
    /// ```
    #[tracing::instrument(level = "error")]
    pub fn from_str(input: &str) -> Result<Self, ControlFileError> {
        let mut temp = HashMap::new();
        for line in input.lines() {
            let parts: Vec<&str> = line.split('=').collect();

            if parts.len() != 2 {
                continue;
            }

            let (k, v) = (parts.get(0).unwrap().trim(), parts.get(1).unwrap().trim());

            let v = v.trim_start_matches('\'');
            let v = v.trim_end_matches('\'');

            temp.insert(k, v);
        }
        Ok(ControlFile {
            comment: temp
                .get("comment")
                .ok_or(ControlFileError::MissingField {
                    field: "comment",
                    context: SpanTrace::capture(),
                })?
                .to_string(),
            default_version: temp
                .get("default_version")
                .ok_or(ControlFileError::MissingField {
                    field: "default_version",
                    context: SpanTrace::capture(),
                })?
                .to_string(),
            module_pathname: temp.get("module_pathname").map(|v| v.to_string()),
            relocatable: temp.get("relocatable").ok_or(ControlFileError::MissingField {
                field: "relocatable",
                context: SpanTrace::capture(),
            })? == &"true",
            superuser: temp.get("superuser").ok_or(ControlFileError::MissingField {
                field: "superuser",
                context: SpanTrace::capture(),
            })? == &"true",
            schema: temp.get("schema").map(|v| v.to_string()),
        })
    }
}

impl From<ControlFile> for SqlGraphEntity {
    fn from(val: ControlFile) -> Self {
        SqlGraphEntity::ExtensionRoot(val)
    }
}

/// An error met while parsing a `.control` file.
#[derive(Debug, Clone)]
pub enum ControlFileError {
    MissingField { field: &'static str, context: SpanTrace },
}

impl std::fmt::Display for ControlFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlFileError::MissingField { field, context } => {
                write!(f, "Missing field in control file! Please add `{}`.", field)?;
                context.fmt(f)?;
            }
        };
        Ok(())
    }
}

impl std::error::Error for ControlFileError {}

impl TryFrom<&str> for ControlFile {
    type Error = ControlFileError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        Self::from_str(input)
    }
}

impl ToSql for ControlFile {
    #[tracing::instrument(level = "debug", err, skip(self, _context))]
    fn to_sql(&self, _context: &super::OgxSql) -> eyre::Result<String> {
        let sql = format!(
            "\
            /* \n\
            This file is auto generated by ogx.\n\
            \n\
            The ordering of items is not stable, it is driven by a dependency graph.\n\
            */\
        "
        );
        tracing::trace!(%sql);
        Ok(sql)
    }
}

impl SqlGraphIdentifier for ControlFile {
    fn dot_identifier(&self) -> String {
        format!("extension root")
    }
    fn rust_identifier(&self) -> String {
        format!("root")
    }

    fn file(&self) -> Option<&'static str> {
        None
    }

    fn line(&self) -> Option<u32> {
        None
    }
}