use crate::symbols::CRATE;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Error, Ident, Path, Token,
};

pub struct Inject {
    pub crate_path: Path,
}

impl Parse for Inject {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                crate_path: parse_quote! {::coi_actix_web},
            });
        }
        let ident: Ident = input.parse()?;
        if ident != CRATE {
            return Err(Error::new(input.span(), "expected `crate` or no params"));
        }

        let _eq: Token![=] = input.parse()?;
        let crate_path = input.parse()?;
        if input.is_empty() {
            Ok(Self { crate_path })
        } else {
            Err(Error::new(
                input.span(),
                "unexpected tokens at the end of crate field attribute",
            ))
        }
    }
}
