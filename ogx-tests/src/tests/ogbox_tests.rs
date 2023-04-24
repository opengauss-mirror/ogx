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
    use ogx::{AllocatedByRust, OgMemoryContexts};

    #[og_test]
    fn ogbox_alloc() {
        let mut ptr: OgBox<i32, AllocatedByRust> = OgBox::<i32>::alloc();
        // ptr is uninitialized data!!! This is dangerous to read from!!!
        *ptr = 5;

        assert_eq!(*ptr, 5);
    }

    #[og_test]
    fn ogbox_alloc0() {
        let mut ptr: OgBox<i32, AllocatedByRust> = OgBox::<i32>::alloc0();

        assert_eq!(*ptr, 0);

        *ptr = 5;

        assert_eq!(*ptr, 5);
    }

    #[og_test]
    fn ogbox_new() {
        let ptr: OgBox<i32, AllocatedByRust> = OgBox::new(5);
        assert_eq!(*ptr, 5);

        let mut ptr: OgBox<Vec<i32>, AllocatedByRust> = OgBox::new(vec![]);
        assert_eq!(*ptr, Vec::<i32>::default());

        ptr.push(1);
        assert_eq!(*ptr, vec![1]);

        ptr.push(2);
        assert_eq!(*ptr, vec![1, 2]);

        ptr.push(3);
        assert_eq!(*ptr, vec![1, 2, 3]);

        let drained = ptr.drain(..).collect::<Vec<_>>();
        assert_eq!(drained, vec![1, 2, 3])
    }

    #[og_test]
    fn ogbox_new_in_context() {
        let ptr: OgBox<i32, AllocatedByRust> =
            OgBox::new_in_context(5, OgMemoryContexts::CurrentMemoryContext);
        assert_eq!(*ptr, 5);

        let mut ptr: OgBox<Vec<i32>, AllocatedByRust> =
        OgBox::new_in_context(vec![], OgMemoryContexts::CurrentMemoryContext);
        assert_eq!(*ptr, Vec::<i32>::default());

        ptr.push(1);
        assert_eq!(*ptr, vec![1]);

        ptr.push(2);
        assert_eq!(*ptr, vec![1, 2]);

        ptr.push(3);
        assert_eq!(*ptr, vec![1, 2, 3]);

        let drained = ptr.drain(..).collect::<Vec<_>>();
        assert_eq!(drained, vec![1, 2, 3])
    }
}
