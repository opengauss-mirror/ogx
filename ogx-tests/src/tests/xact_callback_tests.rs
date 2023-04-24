/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/

#[cfg(any(test, feature = "og_test"))]
#[ogx::og_schema]
mod tests {
    #[allow(unused_imports)]
    use crate as ogx_tests;

    use ogx::prelude::*;
    use ogx::{info, register_xact_callback, OgXactCallbackEvent};

    #[test]
    fn make_idea_happy() {}

    #[og_test]
    fn test_xact_callback() {
        register_xact_callback(OgXactCallbackEvent::Abort, || info!("TESTMSG: Called on abort"));
    }
}
