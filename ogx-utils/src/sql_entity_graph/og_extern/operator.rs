/*
Portions Copyright 2019-2021 ZomboDB, LLC.
Portions Copyright 2021-2022 Technology Concepts & Design, Inc. <support@tcdi.com>

All rights reserved.

Use of this source code is governed by the MIT license that can be found in the LICENSE file.
*/
/*!

`#[og_operator]` related macro expansion for Rust to SQL translation

> Like all of the [`sql_entity_graph`][crate::sql_entity_graph] APIs, this is considered **internal**
to the `ogx` framework and very subject to change between versions. While you may use this, please do it with caution.

*/
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parenthesized;
use syn::parse::{Parse, ParseBuffer};
use syn::token::Paren;

/// A parsed `#[og_operator]` operator.
///
/// It is created during [`OgExtern`](crate::sql_entity_graph::OgExtern) parsing.
#[derive(Debug, Default, Clone)]
pub struct OgOperator {
    pub opname: Option<OgxOperatorOpName>,
    pub commutator: Option<OgxOperatorAttributeWithIdent>,
    pub negator: Option<OgxOperatorAttributeWithIdent>,
    pub restrict: Option<OgxOperatorAttributeWithIdent>,
    pub join: Option<OgxOperatorAttributeWithIdent>,
    pub hashes: bool,
    pub merges: bool,
}

impl ToTokens for OgOperator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let opname = self.opname.iter().clone();
        let commutator = self.commutator.iter().clone();
        let negator = self.negator.iter().clone();
        let restrict = self.restrict.iter().clone();
        let join = self.join.iter().clone();
        let hashes = self.hashes;
        let merges = self.merges;
        let quoted = quote! {
            ::ogx::utils::sql_entity_graph::OgOperatorEntity {
                opname: None #( .unwrap_or(Some(#opname)) )*,
                commutator: None #( .unwrap_or(Some(#commutator)) )*,
                negator: None #( .unwrap_or(Some(#negator)) )*,
                restrict: None #( .unwrap_or(Some(#restrict)) )*,
                join: None #( .unwrap_or(Some(#join)) )*,
                hashes: #hashes,
                merges: #merges,
            }
        };
        tokens.append_all(quoted);
    }
}

#[derive(Debug, Clone)]
pub struct OgxOperatorAttributeWithIdent {
    pub paren_token: Paren,
    pub fn_name: TokenStream2,
}

impl Parse for OgxOperatorAttributeWithIdent {
    fn parse(input: &ParseBuffer) -> Result<Self, syn::Error> {
        let inner;
        Ok(OgxOperatorAttributeWithIdent {
            paren_token: parenthesized!(inner in input),
            fn_name: inner.parse()?,
        })
    }
}

impl ToTokens for OgxOperatorAttributeWithIdent {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fn_name = &self.fn_name;
        let operator = fn_name.to_string().replace(" ", "");
        let quoted = quote! {
            #operator
        };
        tokens.append_all(quoted);
    }
}

#[derive(Debug, Clone)]
pub struct OgxOperatorOpName {
    pub paren_token: Paren,
    pub op_name: TokenStream2,
}

impl Parse for OgxOperatorOpName {
    fn parse(input: &ParseBuffer) -> Result<Self, syn::Error> {
        let inner;
        Ok(OgxOperatorOpName {
            paren_token: parenthesized!(inner in input),
            op_name: inner.parse()?,
        })
    }
}

impl ToTokens for OgxOperatorOpName {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let op_name = &self.op_name;
        let op_string = op_name.to_string().replacen(" ", "", 256);
        let quoted = quote! {
            #op_string
        };
        tokens.append_all(quoted);
    }
}
