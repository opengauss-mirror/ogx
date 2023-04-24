/*!

`#[og_trigger]` related macro expansion for Rust to SQL translation

> Like all of the [`sql_entity_graph`][crate::sql_entity_graph] APIs, this is considered **internal**
to the `ogx` framework and very subject to change between versions. While you may use this, please do it with caution.

*/
pub mod attribute;
pub mod entity;

use crate::sql_entity_graph::ToSqlConfig;
use attribute::OgTriggerAttribute;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{ItemFn, Token};

#[derive(Debug, Clone)]
pub struct OgTrigger {
    func: syn::ItemFn,
    to_sql_config: ToSqlConfig,
}

impl OgTrigger {
    pub fn new(
        func: ItemFn,
        attributes: syn::punctuated::Punctuated<OgTriggerAttribute, Token![,]>,
    ) -> Result<Self, syn::Error> {
        if attributes.len() > 1 {
            return Err(syn::Error::new(
                Span::call_site(),
                "Multiple `sql` arguments found, it must be unique",
            ));
        };
        let to_sql_config = attributes
            .first()
            .cloned()
            .map(|OgTriggerAttribute::Sql(mut config)| {
                if let Some(ref mut content) = config.content {
                    let value = content.value();
                    let updated_value = value
                        .replace("@FUNCTION_NAME@", &*(func.sig.ident.to_string() + "_wrapper"))
                        + "\n";
                    *content = syn::LitStr::new(&updated_value, Span::call_site());
                };
                config
            })
            .unwrap_or_default();

        if !to_sql_config.overrides_default() {
            crate::ident_is_acceptable_to_opengauss(&func.sig.ident)?;
        }

        Ok(Self { func, to_sql_config })
    }

    pub fn entity_tokens(&self) -> Result<ItemFn, syn::Error> {
        let sql_graph_entity_fn_name = syn::Ident::new(
            &format!("__ogx_internals_trigger_{}", self.func.sig.ident.to_string()),
            self.func.sig.ident.span(),
        );
        let func_sig_ident = &self.func.sig.ident;
        let function_name = func_sig_ident.to_string();
        let to_sql_config = &self.to_sql_config;

        let tokens = quote! {
            #[no_mangle]
            #[doc(hidden)]
            pub extern "Rust" fn #sql_graph_entity_fn_name() -> ::ogx::utils::sql_entity_graph::SqlGraphEntity {
                use core::any::TypeId;
                extern crate alloc;
                use alloc::vec::Vec;
                use alloc::vec;
                let submission = ::ogx::utils::sql_entity_graph::OgTriggerEntity {
                    function_name: #function_name,
                    file: file!(),
                    line: line!(),
                    full_path: concat!(module_path!(), "::", stringify!(#func_sig_ident)),
                    module_path: module_path!(),
                    to_sql_config: #to_sql_config,
                };
                ::ogx::utils::sql_entity_graph::SqlGraphEntity::Trigger(submission)
            }
        };
        syn::parse2(tokens)
    }

    pub fn wrapper_tokens(&self) -> Result<ItemFn, syn::Error> {
        let function_ident = &self.func.sig.ident;
        let extern_func_ident = syn::Ident::new(
            &format!("{}_wrapper", self.func.sig.ident.to_string()),
            self.func.sig.ident.span(),
        );
        let tokens = quote! {
            #[no_mangle]
            #[ogx::og_guard]
            extern "C" fn #extern_func_ident(fcinfo: ::ogx::pg_sys::FunctionCallInfo) -> ::ogx::pg_sys::Datum {
                let maybe_og_trigger = unsafe { ::ogx::trigger_support::OgTrigger::from_fcinfo(fcinfo) };
                let og_trigger = maybe_og_trigger.expect("OgTrigger::from_fcinfo failed");
                let trigger_fn_result: Result<
                    ::ogx::heap_tuple::PgHeapTuple<'_, _>,
                    _,
                > = #function_ident(&og_trigger);

                let trigger_retval = trigger_fn_result.expect("Trigger function panic");
                match trigger_retval.into_trigger_datum() {
                    None => ::ogx::pg_return_null(fcinfo),
                    Some(datum) => datum,
                }
            }

        };
        syn::parse2(tokens)
    }

    pub fn finfo_tokens(&self) -> Result<ItemFn, syn::Error> {
        let finfo_name = syn::Ident::new(
            &format!("pg_finfo_{}_wrapper", self.func.sig.ident),
            proc_macro2::Span::call_site(),
        );
        let tokens = quote! {
            #[no_mangle]
            #[doc(hidden)]
            pub extern "C" fn #finfo_name() -> &'static ::ogx::pg_sys::Pg_finfo_record {
                const V1_API: ::ogx::pg_sys::Pg_finfo_record = ::ogx::pg_sys::Pg_finfo_record { api_version: 1 };
                &V1_API
            }
        };
        syn::parse2(tokens)
    }
}

impl ToTokens for OgTrigger {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let entity_func = self.entity_tokens().expect("Generating entity function for trigger");
        let wrapper_func = self.wrapper_tokens().expect("Generating wrappper function for trigger");
        let finfo_func = self.finfo_tokens().expect("Generating finfo function for trigger");
        let func = &self.func;

        let items = quote! {
            #func

            #wrapper_func

            #finfo_func

            #entity_func
        };
        tokens.append_all(items);
    }
}
