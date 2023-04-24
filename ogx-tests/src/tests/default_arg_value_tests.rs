/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use ogx::prelude::*;

#[og_extern]
fn negative_default_argument(i: default!(i32, -1)) -> i32 {
    i
}

#[og_extern]
fn default_argument(a: default!(i32, 99)) -> i32 {
    a
}

#[og_extern]
fn option_default_argument(a: default!(Option<&str>, "NULL")) -> &str {
    match a {
        Some(a) => a,
        None => "got default of null",
    }
}

#[cfg(any(test, feature = "og_test"))]
#[ogx::og_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as ogx_tests;

    use ogx::prelude::*;

    #[test]
    fn make_idea_happy() {}

    #[og_test]
    fn test_negative_default_argument() {
        let result = Spi::get_one::<i32>("SELECT negative_default_argument();")
            .expect("didn't get SPI result");
        assert_eq!(result, -1);
    }

    #[og_test]
    fn test_default_argument() {
        let result =
            Spi::get_one::<i32>("SELECT default_argument();").expect("didn't get SPI result");
        assert_eq!(result, 99);
    }

    #[og_test]
    fn test_default_argument_specified() {
        let result =
            Spi::get_one::<i32>("SELECT default_argument(2);").expect("didn't get SPI result");
        assert_eq!(result, 2);
    }

    #[og_test]
    fn test_option_default_argument() {
        let result = Spi::get_one::<&str>("SELECT option_default_argument();")
            .expect("didn't get SPI result");
        assert_eq!(result, "got default of null");
    }

    #[og_test]
    fn test_option_default_argument_specified() {
        let result = Spi::get_one::<&str>("SELECT option_default_argument('test');")
            .expect("didn't get SPI result");
        assert_eq!(result, "test");
    }
}
