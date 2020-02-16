//! Provides the `inject` proc macro for use by the [`coi-actix-web`] crate.
//!
//! [`coi-actix-web`]: https://docs.rs/coi-actix-web

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, Error, FnArg, GenericArgument, Ident, ItemFn, Pat,
    PathArguments, Result, Type, TypePath,
};

fn get_arc_ty(ty: &Type, type_path: &TypePath) -> Result<Type> {
    let make_arc_error = || Err(Error::new_spanned(ty, "only Arc<...> can be injected"));
    if type_path.path.leading_colon.is_some() || type_path.path.segments.len() != 1 {
        return make_arc_error();
    }
    let segment = &type_path.path.segments[0];
    if segment.ident != "Arc" {
        return make_arc_error();
    }
    let angle_args = match &segment.arguments {
        PathArguments::AngleBracketed(angle_args) => angle_args,
        _ => return make_arc_error(),
    };
    let args = &angle_args.args;
    if args.len() != 1 {
        return make_arc_error();
    }

    if let GenericArgument::Type(ty) = &args[0] {
        Ok(ty.clone())
    } else {
        make_arc_error()
    }
}

/// The #[inject] proc macro should only be applied to functions that will
/// be passed to [`actix-web`]'s routing APIs.
///
/// [`actix-web`]: https://docs.rs/actix-web
///
/// ## Examples
/// ```rust,ignore
/// use coi::inject;
///
/// #[inject]
/// async fn get_all(#[inject] service: Arc<dyn IService>) -> Result<impl Responder, ()> {
///     ...
/// }
/// ```
///
/// This proc macro changes the input arguments to the fn that it's applied to. All `#[inject]` args
/// get collected into a single type and are pattern matched out. This is to take advantage of the
/// [`coi-actix-web`] crate's `FromResponse` impls. By ensuring that all injected types are part of
/// the same type, we can guarantee that all injected types are resolved from the same scoped
/// container. The downside of this is that the signature you see is not what is generated, and
/// this makes manually calling these functions more verbose. Since all of these functions are
/// expected to be passed to [`actix-web`]'s routing APIs, it's not an issue since those are all
/// generic.
///
/// [`coi-actix-web`]: https://docs.rs/coi-actix-web
/// [`actix-web`]: https://docs.rs/actix-web
#[proc_macro_attribute]
pub fn inject(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemFn);
    let sig = &mut input.sig;
    let inputs = &mut sig.inputs;
    let mut args = vec![];
    while inputs.len() > 0 {
        if let Some(arg) = inputs.pop() {
            args.push(arg);
        }
    }
    args.reverse();
    let (inject, not_inject): (Vec<_>, Vec<_>) =
        args.into_iter().partition(|arg| match arg.value() {
            FnArg::Typed(arg) => arg.attrs.iter().any(|attr| attr.path.is_ident("inject")),
            _ => false,
        });

    for arg in not_inject {
        let (arg, punct) = arg.into_tuple();
        inputs.push_value(arg);
        if let Some(punct) = punct {
            inputs.push_punct(punct);
        }
    }

    let num_args = inject.len();
    let (key, ty): (Vec<Result<Ident>>, Vec<Result<Type>>) = inject
        .into_iter()
        .map(|arg| match arg.value() {
            FnArg::Typed(arg) => {
                let pat = match &*arg.pat {
                    Pat::Ident(pat_ident) => {
                        let ident = &pat_ident.ident;
                        Ok(parse_quote! { #ident })
                    }
                    _ => Err(Error::new_spanned(&*arg.pat, "patterns cannot be injected")),
                };

                let ty = if let Type::Path(type_path) = &*arg.ty {
                    get_arc_ty(&*arg.ty, type_path)
                } else {
                    Err(Error::new_spanned(
                        &*arg.ty,
                        "only Arc<...> can be injected",
                    ))
                };
                (pat, ty)
            }
            _ => unreachable!(),
        })
        .unzip();
    let key = match key.into_iter().collect::<Result<Vec<_>>>() {
        Ok(key) => key,
        Err(e) => return e.to_compile_error().into(),
    };
    let ty = match ty.into_iter().collect::<Result<Vec<_>>>() {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };
    let key_str = key.iter().map(|k| format!("{}", k)).collect::<Vec<_>>();

    let injected_arg = if num_args > 1 {
        let injected_n = format_ident!("Injected{}", num_args);
        parse_quote! {
            coi::#injected_n (( #(
                coi::Injected(#key),
            )* )) :
            coi::#injected_n<#( #ty, )* #( #key_str, )*>
        }
    } else {
        parse_quote! {
            coi::Injected(#( #key, )*):
            coi::Injected<#( Arc<#ty>, )* #( #key_str, )*>
        }
    };
    inputs.push(injected_arg);

    let expanded = quote! {
        #input
    };
    TokenStream::from(expanded)
}
