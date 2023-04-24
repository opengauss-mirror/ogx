/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

mod framework;
#[cfg(any(test, feature = "og_test"))]
mod tests;

pub use framework::*;

#[cfg(any(test, feature = "og_test"))]
ogx::pg_sql_graph_magic!();

#[cfg(test)]
pub mod og_test {
    pub fn setup(_options: Vec<&str>) {
        // noop
    }

    pub fn opengauss_conf_options() -> Vec<&'static str> {
        vec![]
    }
}
