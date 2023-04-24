/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
use ogx::prelude::*;
use ogx::WhoAllocated;

ogx::pg_module_magic!();

#[derive(thiserror::Error, Debug)]
enum TriggerError {
    #[error("Null OLD found")]
    NullOld,
    #[error("PgHeapTuple error: {0}")]
    PgHeapTuple(#[from] ogx::heap_tuple::PgHeapTupleError),
    #[error("TryFromDatumError error: {0}")]
    TryFromDatum(#[from] ogx::datum::TryFromDatumError),
    #[error("TryFromInt error: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
}

#[og_trigger]
fn trigger_example(
    trigger: &ogx::OgTrigger,
) -> Result<PgHeapTuple<'_, impl WhoAllocated<ogx::pg_sys::HeapTupleData>>, TriggerError> {
    let old = trigger.current().ok_or(TriggerError::NullOld)?;

    let mut current = old.into_owned();
    let col_name = "title";

    if current.get_by_name(col_name)? == Some("Fox") {
        current.set_by_name(col_name, "Bear")?;
    }

    Ok(current)
}

extension_sql!(
    r#"
CREATE TABLE test (
    id serial8 NOT NULL PRIMARY KEY,
    title varchar(50),
    description text,
    payload jsonb
);

CREATE TRIGGER test_trigger BEFORE INSERT ON test FOR EACH ROW EXECUTE PROCEDURE trigger_example();
INSERT INTO test (title, description, payload) VALUES ('Fox', 'a description', '{"key": "value"}');
"#,
    name = "create_trigger",
    requires = [trigger_example]
);

#[cfg(any(test, feature = "og_test"))]
#[og_schema]
mod tests {
    use ogx::prelude::*;

    #[og_test]
    fn test_insert() {
        Spi::run(
            r#"INSERT INTO test (title, description, payload) VALUES ('a different title', 'a different description', '{"key": "value"}')"#,
        );
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
