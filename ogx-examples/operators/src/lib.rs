/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use ogx::prelude::*;
use ogx::{opname, og_operator};
use serde::{Deserialize, Serialize};
mod derived;

ogx::pg_module_magic!();

#[derive(OgType, Serialize, Deserialize, Eq, PartialEq)]
pub struct MyType {
    value: i32,
}

#[og_operator]
#[opname(=)]
fn my_eq(left: MyType, right: MyType) -> bool {
    left == right
}
