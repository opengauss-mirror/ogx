/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

mod complex;
mod fixed_size;
mod generic_enum;
mod hstore_clone;

ogx::pg_module_magic!();

#[cfg(test)]
#[ogx::og_schema]
pub mod og_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the og_test framework starts
    }

    pub fn () -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
