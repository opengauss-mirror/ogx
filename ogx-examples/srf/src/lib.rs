/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use ogx::prelude::*;

ogx::pg_module_magic!();

#[og_extern]
fn generate_series(start: i64, finish: i64, step: default!(i64, 1)) -> SetOfIterator<'static, i64> {
    SetOfIterator::new((start..=finish).step_by(step as usize))
}

#[og_extern]
fn random_values(num_rows: i32) -> TableIterator<'static, (name!(index, i32), name!(value, f64))> {
    TableIterator::new((1..=num_rows).map(|i| (i, rand::random::<f64>())))
}

#[og_extern]
fn vector_of_static_values() -> SetOfIterator<'static, &'static str> {
    let values = vec!["Brandy", "Sally", "Anchovy"];
    SetOfIterator::new(values.into_iter())
}

#[og_extern]
fn return_tuple(
) -> TableIterator<'static, (name!(id, i32), name!(name, &'static str), name!(age, f64))> {
    TableIterator::once((1, "Brandy", 4.5))
}

#[cfg(any(test, feature = "og_test"))]
#[og_schema]
mod tests {
    use ogx::prelude::*;

    #[og_test]
    fn test_it() {
        // do testing here.
        //
        // #[og_test] functions run *inside* Postgres and have access to all Postgres internals
        //
        // Normal #[test] functions do not
        //
        // In either case, they all run in parallel
    }
}

#[cfg(test)]
pub mod og_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the og_test framework starts
    }

    pub fn opengauss_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
