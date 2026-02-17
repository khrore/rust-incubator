use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Expr, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

struct Entry {
    key: Expr,
    value: Expr,
}

impl Parse for Entry {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let key = input.parse::<Expr>()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse::<Expr>()?;
        Ok(Self { key, value })
    }
}

struct Entries {
    entries: Punctuated<Entry, Token![,]>,
}

impl Parse for Entries {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let entries = Punctuated::parse_terminated(input)?;
        Ok(Self { entries })
    }
}

#[proc_macro]
pub fn btreemap(input: TokenStream) -> TokenStream {
    let entries = parse_macro_input!(input as Entries);
    let insertions = entries.entries.iter().map(|entry| {
        let key = &entry.key;
        let value = &entry.value;
        quote! {
            __btreemap.insert(#key, #value);
        }
    });

    quote! {
        {
            let mut __btreemap = ::std::collections::BTreeMap::new();
            #(#insertions)*
            __btreemap
        }
    }
    .into()
}
