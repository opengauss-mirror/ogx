/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
//! This exists just to make sure we can compile various things under `#![no_std]`

#![no_std]
extern crate alloc;

use ogx::*;
use serde::{Deserialize, Serialize};

use alloc::string::String;

ogx::pg_module_magic!();

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

#[derive(OgType, Serialize, Deserialize, Eq, PartialEq)]
pub struct MyType {
    value: i32,
}

#[og_operator]
#[opname(=)]
fn my_eq(left: MyType, right: MyType) -> bool {
    left == right
}

#[og_extern]
fn hello_nostd() -> &'static str {
    "Hello, nostd"
}

#[og_extern]
fn echo(input: String) -> String {
    input
}
