/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
use ogx::*;
use serde::{Deserialize, Serialize};

/// standard Rust equality/comparison derives
#[derive(Eq, PartialEq, Ord, Hash, PartialOrd)]

/// Support using this struct as a Postgres type, which the easy way requires Serde
#[derive(OgType, Serialize, Deserialize)]

/// automatically generate =, <> SQL operator functions
#[derive(OgEq)]

/// automatically generate <, >, <=, >=, and a "_cmp" SQL functions
/// When "OgEq" is also derived, ogx also creates an "opclass" (and family)
/// so that the type can be used in indexes `USING btree`
#[derive(OgOrd)]

/// automatically generate a "_hash" function, and the necessary "opclass" (and family)
/// so the type can also be used in indexes `USING hash`
#[derive(OgHash)]
pub struct Thing(String);

// and there's no code to write!
