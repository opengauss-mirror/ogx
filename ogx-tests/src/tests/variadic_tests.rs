/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

#[ogx::og_schema]
mod test {
    use ogx::prelude::*;
    use ogx::VariadicArray;

    #[og_extern]
    fn func_with_variadic_array_args<'a>(
        _field: &'a str,
        values: VariadicArray<&'a str>,
    ) -> String {
        values.get(0).unwrap().unwrap().to_string()
    }
}

#[cfg(any(test, feature = "og_test"))]
#[ogx::og_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as ogx_tests;

    use ogx::prelude::*;

    #[og_test]
    fn test_func_with_variadic_array_args() {
        let result = Spi::get_one::<&str>(
            "SELECT test.func_with_variadic_array_args('test', 'a', 'b', 'c');",
        )
        .expect("didn't get SPI result");
        assert_eq!(result, "a");
    }
}
