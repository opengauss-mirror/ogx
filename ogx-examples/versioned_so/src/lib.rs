use ogx::prelude::*;

ogx::pg_module_magic!();

#[og_extern]
fn hello_versioned_so() -> &'static str {
    "Hello, versioned_so"
}

#[cfg(any(test, feature = "og_test"))]
#[og_schema]
mod tests {
    use ogx::prelude::*;

    #[og_test]
    fn test_hello_versioned_so() {
        assert_eq!("Hello, versioned_so", crate::hello_versioned_so());
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
