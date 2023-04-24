/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
/// All top-level ogx objects, **regardless** of the ".rs" file they're defined in, are created
/// in the schema determined by `CREATE EXTENSION`.  It could be `public` (the default), or a
/// user-specified schema. We have no idea what that is.
use ogx::prelude::*;
use serde::{Deserialize, Serialize};

ogx::pg_module_magic!();

#[derive(OgType, Serialize, Deserialize)]
pub struct MyType(pub(crate) String);

#[og_extern]
fn hello_default_schema() -> &'static str {
    "Hello from the schema where you installed this extension"
}

/// we can create our own schemas, which are just Rust `mod`s.  Anything defined in this module
/// will be created in a Postgres schema of the same name
#[og_schema]
mod some_schema {
    use ogx::*;
    use serde::{Deserialize, Serialize};

    #[derive(OgType, Serialize, Deserialize)]
    pub struct MySomeSchemaType(pub(crate) String);

    #[og_extern]
    fn hello_some_schema() -> &'static str {
        "Hello from some_schema"
    }
}

/// we can also cheat and put ogx objects in Postgres' `pg_catalog` schema,
/// which will make them available regardless of the active `search_path`, but
/// requires that the extension be created by a super-user
#[og_schema]
mod pg_catalog {
    use ogx::*;
    use serde::{Deserialize, Serialize};

    #[derive(OgType, Serialize, Deserialize)]
    pub struct MyPgCatalogType(pub(crate) String);
}

/// similarly, we can create objects in Postgres' `public` schema.  This will at least require the
/// proper permissions by the user calling `CREATE EXTENSION`
#[og_schema]
mod public {
    use ogx::prelude::*;

    #[og_extern]
    pub fn hello_public() -> &'static str {
        "Hello from the public schema"
    }
}

#[cfg(any(test, feature = "og_test"))]
#[og_schema]
mod tests {
    use crate::pg_catalog::MyPgCatalogType;
    use crate::some_schema::MySomeSchemaType;
    use crate::MyType;
    use ogx::prelude::*;

    #[og_test]
    fn test_hello_default_schema() {
        assert_eq!(
            "Hello from the schema where you installed this extension",
            Spi::get_one::<&str>("SELECT hello_default_schema()").expect("SPI result was NULL")
        );
    }

    #[og_test]
    fn test_my_type() {
        assert_eq!(
            "test",
            // we don't need to qualify "MyType" because whatever schema it was created in
            // is applied to the "search_path" of this test function
            Spi::get_one::<MyType>("SELECT '\"test\"'::MyType").expect("SPI reault was NULL").0
        );
    }

    #[og_test]
    fn test_hello_some_schema() {
        assert_eq!(
            "Hello from some_schema",
            // "hello_some_schema()" is in "some_schema", so it needs to be qualified
            Spi::get_one::<&str>("SELECT some_schema.hello_some_schema()")
                .expect("SPI result was NULL")
        );
    }

    #[og_test]
    fn test_my_some_schema_type() {
        assert_eq!(
            String::from("test"),
            // "MySomeSchemaType" is in 'some_schema', so it needs to be qualified
            Spi::get_one::<MySomeSchemaType>("SELECT '\"test\"'::some_schema.MySomeSchemaType")
                .expect("SPI result was NULL")
                .0
        )
    }

    #[og_test]
    fn test_my_pg_catalog_type() {
        assert_eq!(
            String::from("test"),
            Spi::get_one::<MyPgCatalogType>("SELECT '\"test\"'::MyPgCatalogType")
                .expect("SPI result was NULL")
                .0
        )
    }

    #[og_test]
    fn test_hello_public() {
        assert_eq!(
            "Hello from the public schema",
            Spi::get_one::<&str>("SELECT hello_public()").expect("SPI result was NULL")
        )
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