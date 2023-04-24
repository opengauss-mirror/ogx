/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

use ogx::prelude::*;
use ogx::{InOutFuncs, StringInfo};
use serde::{Deserialize, Serialize};

#[og_extern]
fn add_two_numbers(a: i32, b: i32) -> i32 {
    a + b
}

#[og_extern]
fn takes_i16(i: i16) -> i16 {
    i
}

#[og_extern]
fn takes_i32(i: i32) -> i32 {
    i
}

#[og_extern]
fn takes_i64(i: i32) -> i32 {
    i
}

#[og_extern]
fn takes_bool(i: bool) -> bool {
    i
}

#[og_extern]
fn takes_f32(i: f32) -> f32 {
    i
}

#[og_extern]
fn takes_f64(i: f64) -> f64 {
    i
}

#[og_extern]
fn takes_i8(i: i8) -> i8 {
    i
}

#[og_extern]
fn takes_char(i: char) -> char {
    i
}

#[og_extern]
fn takes_option(i: Option<i32>) -> i32 {
    match i {
        Some(i) => i,
        None => -1,
    }
}

#[og_extern]
fn takes_str(s: &str) -> &str {
    s
}

#[og_extern]
fn takes_string(s: String) -> String {
    s
}

#[og_extern]
fn returns_some() -> Option<i32> {
    Some(42)
}

#[og_extern]
fn returns_none() -> Option<i32> {
    None
}

#[og_extern]
fn returns_void() {
    // noop
}

#[og_extern]
fn returns_tuple() -> TableIterator<'static, (name!(id, i32), name!(title, String))> {
    TableIterator::once((42, "ogx".into()))
}

#[og_extern]
fn same_name(same_name: &str) -> &str {
    same_name
}

// Tests for regression of https://github.com/zombodb/pgx/issues/432
#[og_extern]
fn fcinfo_renamed_one_arg(
    _x: OgBox<pg_sys::IndexAmRoutine>,
    _fcinfo: pg_sys::FunctionCallInfo,
) -> OgBox<pg_sys::IndexAmRoutine> {
    todo!()
}

#[og_extern]
fn fcinfo_renamed_no_arg(_fcinfo: pg_sys::FunctionCallInfo) -> i32 {
    todo!()
}

#[og_extern]
fn fcinfo_not_named_one_arg(
    _x: OgBox<pg_sys::IndexAmRoutine>,
    fcinfo: pg_sys::FunctionCallInfo,
) -> OgBox<pg_sys::IndexAmRoutine> {
    let _fcinfo = fcinfo;
    todo!()
}

#[og_extern]
fn fcinfo_not_named_no_arg(fcinfo: pg_sys::FunctionCallInfo) -> i32 {
    let _fcinfo = fcinfo;
    todo!()
}

#[derive(OgType, Serialize, Deserialize, Debug, PartialEq)]
#[inoutfuncs]
pub struct NullStrict {}

impl InOutFuncs for NullStrict {
    fn input(_input: &ogx::cstr_core::CStr) -> Self
    where
        Self: Sized,
    {
        NullStrict {}
    }

    fn output(&self, _buffer: &mut StringInfo) {}
    // doesn't define a NULL_ERROR_MESSAGE
}

#[derive(OgType, Serialize, Deserialize, Debug, PartialEq)]
#[inoutfuncs]
pub struct NullError {}

impl InOutFuncs for NullError {
    fn input(_input: &ogx::cstr_core::CStr) -> Self
    where
        Self: Sized,
    {
        NullError {}
    }

    fn output(&self, _buffer: &mut StringInfo) {}

    const NULL_ERROR_MESSAGE: Option<&'static str> = Some("An error message");
}

#[cfg(any(test, feature = "og_test"))]
#[ogx::og_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as ogx_tests;

    use crate::tests::fcinfo_tests::{same_name, NullError, NullStrict};
    use ogx::prelude::*;
    use ogx::{direct_og_extern_function_call, IntoDatum};

    #[test]
    fn make_idea_happy() {
        assert_eq!(5, 5);
    }

    #[og_test]
    fn test_add_two_numbers() {
        assert_eq!(super::add_two_numbers(2, 3), 5);
    }

    #[og_test]
    unsafe fn test_takes_i16() {
        let input = 42i16;
        let result = direct_og_extern_function_call::<i16>(
            super::takes_i16_wrapper,
            vec![input.into_datum()],
        );
        let result = result.expect("result is NULL");
        assert_eq!(result, input);
    }

    #[og_test]
    unsafe fn test_takes_i32() {
        let input = 42i32;
        let result = direct_og_extern_function_call::<i32>(
            super::takes_i32_wrapper,
            vec![input.into_datum()],
        );
        let result = result.expect("result is NULL");
        assert_eq!(result, input);
    }

    #[og_test]
    unsafe fn test_takes_i64() {
        let input = 42i64;
        let result = direct_og_extern_function_call::<i64>(
            super::takes_i64_wrapper,
            vec![input.into_datum()],
        );
        let result = result.expect("result is NULL");
        assert_eq!(result, input);
    }

    #[og_test]
    unsafe fn test_takes_bool() {
        let input = true;
        let result = direct_og_extern_function_call::<bool>(
            super::takes_bool_wrapper,
            vec![input.into_datum()],
        );
        let result = result.expect("result is NULL");
        assert_eq!(result, input);
    }

    #[og_test]
    unsafe fn test_takes_f32() {
        let input = 42.424_244;
        let result = direct_og_extern_function_call::<f32>(
            super::takes_f32_wrapper,
            vec![input.into_datum()],
        );
        let result = result.expect("result is NULL");
        assert!(result.eq(&input));
    }

    #[og_test]
    unsafe fn test_takes_f64() {
        let input = 42.424_242_424_242f64;
        let result = direct_og_extern_function_call::<f64>(
            super::takes_f64_wrapper,
            vec![input.into_datum()],
        );
        let result = result.expect("result is NULL");
        assert!(result.eq(&input));
    }

    #[og_test]
    unsafe fn test_takes_i8() {
        let result = Spi::get_one::<i8>("SELECT takes_i8('a');").expect("SPI result was NULL");
        assert_eq!(result, 'a' as i8);
    }

    #[og_test]
    unsafe fn test_takes_char() {
        let result = Spi::get_one::<char>("SELECT takes_char('🚨');").expect("SPI result was NULL");
        assert_eq!(result, '🚨');
    }

    #[og_test]
    unsafe fn test_takes_option_with_null_arg() {
        let result = direct_og_extern_function_call::<i32>(super::takes_option_wrapper, vec![None]);
        assert_eq!(-1, result.expect("result is NULL"))
    }

    #[og_test]
    unsafe fn test_takes_option_with_non_null_arg() {
        let input = 42i32;
        let result = direct_og_extern_function_call::<i32>(
            super::takes_option_wrapper,
            vec![input.into_datum()],
        );
        let result = result.expect("result is NULL");
        assert_eq!(result, input);
    }

    #[og_test]
    unsafe fn test_takes_str() {
        let input = "this is a test";
        let result = direct_og_extern_function_call::<&str>(
            super::takes_str_wrapper,
            vec![input.into_datum()],
        );
        let result = result.expect("result is NULL");
        assert_eq!(result, input);
    }

    #[og_test]
    unsafe fn test_takes_string() {
        let input = "this is a test".to_string();
        let result = direct_og_extern_function_call::<String>(
            super::takes_str_wrapper,
            vec![input.clone().into_datum()],
        );
        let result = result.expect("result is NULL");
        assert_eq!(result, input);
    }

    #[og_test]
    unsafe fn test_returns_some() {
        let result = direct_og_extern_function_call::<i32>(super::returns_some_wrapper, vec![]);
        assert!(result.is_some());
    }

    #[og_test]
    unsafe fn test_returns_none() {
        let result = direct_og_extern_function_call::<i32>(super::returns_none_wrapper, vec![]);
        assert!(result.is_none())
    }

    #[og_test]
    fn test_returns_void() {
        let result = Spi::get_one::<()>("SELECT returns_void();");
        assert_eq!(result, None)
    }

    #[og_test]
    fn test_returns_tuple() {
        let result = Spi::get_two::<i32, String>("SELECT * FROM returns_tuple();");
        assert_eq!((Some(42), Some("ogx".into())), result)
    }

    /// ensures that we can have a `#[og_extern]` function with an argument that
    /// shares its name
    #[og_test]
    fn test_same_name() {
        assert_eq!("test", same_name("test"));
    }
    #[og_test]
    fn test_null_strict_type() {
        assert_eq!(None, Spi::get_one::<NullStrict>("SELECT null::NullStrict"));
    }
    #[og_test]
    #[should_panic(expected = "An error message")]
    fn test_null_error_type() {
        assert_eq!(None, Spi::get_one::<NullError>("SELECT null::NullError"));
    }
}
