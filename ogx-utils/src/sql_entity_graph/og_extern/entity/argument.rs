/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
/*!

`#[og_extern]` related argument entities for Rust to SQL translation

> Like all of the [`sql_entity_graph`][crate::sql_entity_graph] APIs, this is considered **internal**
to the `ogx` framework and very subject to change between versions. While you may use this, please do it with caution.

*/
use crate::sql_entity_graph::{SqlGraphIdentifier, UsedTypeEntity};

/// The output of a [`OgExternArgument`](crate::sql_entity_graph::OgExternArgument) from `quote::ToTokens::to_tokens`.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OgExternArgumentEntity {
    pub pattern: &'static str,
    pub used_ty: UsedTypeEntity,
}

impl SqlGraphIdentifier for OgExternArgumentEntity {
    fn dot_identifier(&self) -> String {
        format!("arg {}", self.rust_identifier())
    }
    fn rust_identifier(&self) -> String {
        self.used_ty.full_path.to_string()
    }

    fn file(&self) -> Option<&'static str> {
        None
    }

    fn line(&self) -> Option<u32> {
        None
    }
}
