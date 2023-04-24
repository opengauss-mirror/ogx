/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
use ogx::prelude::*;

#[og_extern(name = "renamed_func")]
fn func_to_rename() {}

#[cfg(any(test, feature = "og_test"))]
#[ogx::og_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as ogx_tests;

    use ogx::prelude::*;

    #[og_test]
    fn renamed_func() {
        Spi::run("SELECT renamed_func();");
    }
}
