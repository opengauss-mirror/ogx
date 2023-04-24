/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use ogx::prelude::*;

#[og_extern(immutable)]
fn returns_tuple_with_attributes(
) -> TableIterator<'static, (name!(arg, String), name!(arg2, String))> {
    TableIterator::once(("hi".to_string(), "bye".to_string()))
}

// Check we can map a `fdw_handler`
#[og_extern]
fn fdw_handler_return() -> ogx::OgBox<ogx::pg_sys::FdwRoutine> {
    unimplemented!("Not a functional test, just a signature test for SQL generation. Feel free to make a functional test!")
}

#[cfg(any(test, feature = "og_test"))]
#[ogx::og_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as ogx_tests;

    use ogx::prelude::*;

    // exists just to make sure the code compiles -- tickles the bug behind PR #807
    #[og_extern]
    fn returns_named_tuple_with_rust_reserved_keyword<'a>(/*
                                 `type` is a reserved Rust keyword, but we still need to be able to parse it for SQL generation 
                                                        */
    ) -> TableIterator<'a, (name!(type, String), name!(i, i32))> {
        unimplemented!()
    }

    #[og_extern(immutable)]
    fn is_immutable() {}

    #[og_test]
    fn test_immutable() {
        let result = Spi::get_one::<bool>(
            "SELECT provolatile = 'i' FROM pg_proc WHERE proname = 'is_immutable'",
        )
        .expect("failed to get SPI result");
        assert!(result)
    }

    // Ensures `@MODULE_PATHNAME@` and `@FUNCTION_NAME@` are handled.
    #[og_extern(sql = r#"
        CREATE FUNCTION tests."overridden_sql_with_fn_name"() RETURNS void
        STRICT
        LANGUAGE c /* Rust */
        AS '@MODULE_PATHNAME@', '@FUNCTION_NAME@';
    "#)]
    fn overridden_sql_with_fn_name() -> bool {
        true
    }

    #[og_test]
    fn test_overridden_sql_with_fn_name() {
        let result = Spi::get_one::<bool>(r#"SELECT tests."overridden_sql_with_fn_name"()"#)
            .expect("failed to get SPI result");
        assert!(result)
    }

    // Manually define the function first here. Note that it returns false
    #[og_extern(sql = r#"
        CREATE FUNCTION tests."create_or_replace_method"() RETURNS bool
        STRICT
        LANGUAGE c /* Rust */
        AS '@MODULE_PATHNAME@', '@FUNCTION_NAME@';
    "#)]
    fn create_or_replace_method_first() -> bool {
        false
    }

    // Replace the "create_or_replace_method" function using og_extern[create_or_replace]
    // Note that it returns true, and that we use a "requires = []" here to ensure
    // that there is an order of creation.
    #[og_extern(create_or_replace, requires = [create_or_replace_method_first])]
    fn create_or_replace_method() -> bool {
        true
    }

    // This will test to make sure that a function is created if it doesn't exist
    // while using og_extern[create_or_replace]
    #[og_extern(create_or_replace)]
    fn create_or_replace_method_other() -> i32 {
        42
    }

    #[og_test]
    fn test_create_or_replace() {
        let replace_result = Spi::get_one::<bool>(r#"SELECT tests."create_or_replace_method"()"#)
            .expect("failed to get SPI result");
        assert!(replace_result);

        let create_result =
            Spi::get_one::<i32>(r#"SELECT tests."create_or_replace_method_other"()"#)
                .expect("failed to get SPI result");
        assert_eq!(create_result, 42);
    }

    #[og_extern]
    fn anyele_type(x: ogx::AnyElement) -> i32 {
        x.oid() as i32
    }

    #[og_test]
    fn test_anyele_type() {
        let interval_type =
            Spi::get_one::<i32>(r#"SELECT tests."anyele_type"('5 hours'::interval)"#)
                .expect("failed to get SPI result");
        assert_eq!(interval_type as u32, pg_sys::INTERVALOID);
    }
}
