use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Path, TraitItemMethod, Visibility};

use super::iid::IID;

pub struct Interface {
    pub iid: IID,
    pub visibility: Visibility,
    pub name: Ident,
    pub parent: Option<Path>,
    pub items: Vec<TraitItemMethod>,
    docs: Vec<Attribute>,
}

impl Interface {
    pub fn to_struct_tokens(&self) -> TokenStream {
        let vis = &self.visibility;
        let name = &self.name;
        let vptr = super::vptr::ident(&name);
        let docs = &self.docs;
        quote! {
            #(#docs)*
            #[repr(transparent)]
            #[derive(Copy, Clone, Debug)]
            #vis struct #name {
                inner: ::std::ptr::NonNull<#vptr>,
            }
        }
    }

    pub fn to_iid_tokens(&self) -> TokenStream {
        self.iid.to_tokens(&self.name)
    }

    pub fn is_iunknown(&self) -> bool {
        self.parent.is_none()
    }
}

impl syn::parse::Parse for Interface {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let mut iid = None;
        let mut docs = Vec::new();
        for attr in attributes.into_iter() {
            let path = &attr.path;
            let tokens = &attr.tokens;
            if path.is_ident("doc") {
                docs.push(attr);
            } else if path.is_ident("uuid") {
                let iid_str: ParenthsizedStr = syn::parse2(tokens.clone())?;

                iid = Some(IID::parse(&iid_str.lit)?);
            } else {
                return Err(syn::Error::new(
                    path.span().clone(),
                    format!("Unrecognized attribute '{}'", path.to_token_stream()),
                ));
            }
        }

        let visibility = input.parse::<syn::Visibility>()?;
        let _ = input.parse::<syn::Token![unsafe]>()?;
        let interface = input.parse::<keywords::interface>()?;
        let iid = match iid {
            Some(iid) => iid,
            None => {
                return Err(syn::Error::new(
                    interface.span(),
                    "Interfaces must have a '#[uuid(\"$IID\")]' attribute",
                ))
            }
        };
        let name = input.parse::<Ident>()?;
        let mut parent = None;
        if name.to_string() != "IUnknown" {
            let _ = input.parse::<syn::Token![:]>().map_err(|_| {
                syn::Error::new(
                    name.span(),
                    format!("Interfaces must inherit from another interface like so: `interface {}: IParentInterface`", name),
                )
            })?;
            parent = Some(input.parse::<Path>()?);
        }
        let content;
        syn::braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            items.push(content.parse()?);
        }
        Ok(Self {
            iid,
            visibility,
            items,
            name,
            parent,
            docs,
        })
    }
}

mod keywords {
    syn::custom_keyword!(interface);
}

struct ParenthsizedStr {
    lit: syn::LitStr,
}

impl syn::parse::Parse for ParenthsizedStr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit;
        syn::parenthesized!(lit in input);
        let lit = lit
            .parse()
            .map_err(|e| syn::Error::new(e.span(), format!("uuids must be string literals")))?;

        Ok(Self { lit })
    }
}
