/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
/*!

`#[derive(OgOrd)]` related macro expansion for Rust to SQL translation

> Like all of the [`sql_entity_graph`][crate::sql_entity_graph] APIs, this is considered **internal**
to the `ogx` framework and very subject to change between versions. While you may use this, please do it with caution.

*/
pub mod entity;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Ident};

use crate::sql_entity_graph::ToSqlConfig;

/// A parsed `#[derive(OgOrd)]` item.
///
/// It should be used with [`syn::parse::Parse`] functions.
///
/// Using [`quote::ToTokens`] will output the declaration for a `ogx::datum::sql_entity_graph::InventoryOgOrd`.
///
/// On structs:
///
/// ```rust
/// use syn::{Macro, parse::Parse, parse_quote, parse};
/// use quote::{quote, ToTokens};
/// use ogx_utils::sql_entity_graph::OgOrd;
///
/// # fn main() -> eyre::Result<()> {
/// let parsed: OgOrd = parse_quote! {
///     #[derive(OgOrd)]
///     struct Example<'a> {
///         demo: &'a str,
///     }
/// };
/// let sql_graph_entity_tokens = parsed.to_token_stream();
/// # Ok(())
/// # }
/// ```
///
/// On enums:
///
/// ```rust
/// use syn::{Macro, parse::Parse, parse_quote, parse};
/// use quote::{quote, ToTokens};
/// use ogx_utils::sql_entity_graph::OgOrd;
///
/// # fn main() -> eyre::Result<()> {
/// let parsed: OgOrd = parse_quote! {
///     #[derive(OgOrd)]
///     enum Demo {
///         Example,
///     }
/// };
/// let sql_graph_entity_tokens = parsed.to_token_stream();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct OgOrd {
    pub name: Ident,
    pub to_sql_config: ToSqlConfig,
}

impl OgOrd {
    pub fn new(name: Ident, to_sql_config: ToSqlConfig) -> Result<Self, syn::Error> {
        if !to_sql_config.overrides_default() {
            crate::ident_is_acceptable_to_opengauss(&name)?;
        }

        Ok(Self { name, to_sql_config })
    }

    pub fn from_derive_input(derive_input: DeriveInput) -> Result<Self, syn::Error> {
        let to_sql_config =
            ToSqlConfig::from_attributes(derive_input.attrs.as_slice())?.unwrap_or_default();
        Self::new(derive_input.ident, to_sql_config)
    }
}

impl Parse for OgOrd {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        use syn::Item;

        let parsed = input.parse()?;
        let (ident, attrs) = match &parsed {
            Item::Enum(item) => (item.ident.clone(), item.attrs.as_slice()),
            Item::Struct(item) => (item.ident.clone(), item.attrs.as_slice()),
            _ => return Err(syn::Error::new(input.span(), "expected enum or struct")),
        };
        let to_sql_config = ToSqlConfig::from_attributes(attrs)?.unwrap_or_default();
        Self::new(ident, to_sql_config)
    }
}

impl ToTokens for OgOrd {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let sql_graph_entity_fn_name =
            syn::Ident::new(&format!("__ogx_internals_ord_{}", self.name), Span::call_site());
        let to_sql_config = &self.to_sql_config;
        let inv = quote! {
            #[no_mangle]
            #[doc(hidden)]
            pub extern "Rust" fn  #sql_graph_entity_fn_name() -> ::ogx::utils::sql_entity_graph::SqlGraphEntity {
                use core::any::TypeId;
                extern crate alloc;
                use alloc::vec::Vec;
                use alloc::vec;
                let submission = ::ogx::utils::sql_entity_graph::OgOrdEntity {
                    name: stringify!(#name),
                    file: file!(),
                    line: line!(),
                    full_path: core::any::type_name::<#name>(),
                    module_path: module_path!(),
                    id: TypeId::of::<#name>(),
                    to_sql_config: #to_sql_config,
                };
                ::ogx::utils::sql_entity_graph::SqlGraphEntity::Ord(submission)
            }
        };
        tokens.append_all(inv);
    }
}
