use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, Attribute, FnArg, Item, Signature, Token};

pub mod into_key;
pub mod lazy;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Stream(TokenStream),
}

impl From<Error> for TokenStream {
    fn from(value: Error) -> Self {
        match value {
            Error::Stream(ts) => ts,
        }
    }
}

pub fn generate(item: Item) -> TokenStream {
    inner_generate(item).unwrap_or_else(Into::into)
}
fn is_result_type(output: &syn::ReturnType) -> bool {
    if let syn::ReturnType::Type(_, ty) = output {
        if let syn::Type::Path(type_path) = &**ty {
            // Check if the return type is a Result.
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "Result";
            }
        }
    }
    false
}
fn generate_method(trait_item: &syn::TraitItem) -> Option<TokenStream> {
    if let syn::TraitItem::Fn(method) = trait_item {
        let sig = &method.sig;
        let name = &sig.ident;
        let output = &sig.output;
        let self_ty = get_receiver(sig.inputs.iter().next()?)?;

        let is_result = is_result_type(output);
        let args_without_self = get_args_without_self(&sig.inputs);
        let attrs = &method.attrs;
        let return_question_mark = if is_result { Some(quote!(?)) } else { None };

        if is_mutable_method(self_ty) {
            Some(generate_mutable_method(
                sig,
                attrs,
                name,
                &args_without_self,
                &return_question_mark,
            ))
        } else {
            Some(generate_immutable_method(
                sig,
                attrs,
                name,
                &args_without_self,
            ))
        }
    } else {
        None
    }
}

fn get_receiver(arg: &syn::FnArg) -> Option<&syn::Receiver> {
    if let syn::FnArg::Receiver(receiver) = arg {
        Some(receiver)
    } else {
        None
    }
}

pub fn get_args_without_self(inputs: &Punctuated<FnArg, Token!(,)>) -> Vec<Ident> {
    inputs
        .iter()
        .skip(1)
        .filter_map(|arg| {
            if let syn::FnArg::Typed(syn::PatType { pat, .. }) = arg {
                match &**pat {
                    syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

fn is_mutable_method(receiver: &syn::Receiver) -> bool {
    receiver.reference.is_some() && receiver.mutability.is_some()
}
fn generate_immutable_method(
    sig: &Signature,
    attrs: &[Attribute],
    name: &Ident,
    args_without_self: &[Ident],
) -> TokenStream {
    let inputs = sig.inputs.iter().skip(1);
    let output = &sig.output;
    quote! {
        #(#attrs)*
        fn #name(#(#inputs),*) #output {
            Self::Impl::get_lazy().unwrap_or_default().#name(#(#args_without_self),*)
        }
    }
}

fn generate_mutable_method(
    sig: &Signature,
    attrs: &[Attribute],
    name: &Ident,
    args_without_self: &[Ident],
    return_question_mark: &Option<TokenStream>,
) -> TokenStream {
    let inputs = sig.inputs.iter().skip(1);
    let output = &sig.output;
    let result = if return_question_mark.is_some() {
        quote!(Ok(res))
    } else {
        quote!(res)
    };
    quote! {
        #(#attrs)*
        fn #name(#(#inputs),*) #output {
            let mut impl_ = Self::Impl::get_lazy().unwrap_or_default();
            let res = impl_.#name(#(#args_without_self),*) #return_question_mark;
            Self::Impl::set_lazy(impl_);
            #result
        }
    }
}

fn inner_generate(item: Item) -> Result<TokenStream, Error> {
    if let Item::Trait(input_trait) = &item {
        let generated_methods = input_trait
            .items
            .iter()
            .filter_map(generate_method)
            .collect::<Vec<_>>();

        let trait_ident = &input_trait.ident;
        let new_trait_ident = syn::Ident::new(
            trait_ident.to_string().strip_prefix("Is").ok_or_else(|| {
                Error::Stream(quote! { compile_error!("Trait must start with `Is`"); })
            })?,
            trait_ident.span(),
        );
        let (_, ty_generics, _) = input_trait.generics.split_for_impl();

        let attrs = input_trait.attrs.as_slice();
        let output = quote! {
            #item
            #(#attrs)*
            pub trait #new_trait_ident #ty_generics {
                /// Type that implments the instance type
                type Impl: Lazy + #trait_ident #ty_generics + Default;
                #(#generated_methods)*
            }

        };
        Ok(output)
    } else {
        Err(Error::Stream(
            quote! { compile_error!("Input must be a trait"); },
        ))
    }
}

#[cfg(test)]
mod tests {

    use std::io::{Read, Write};

    /// Format the given snippet. The snippet is expected to be *complete* code.
    /// When we cannot parse the given snippet, this function returns `None`.
    fn format_snippet(snippet: &str) -> String {
        let mut child = std::process::Command::new("rustfmt")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(snippet.as_bytes())
            .map_err(p_e)
            .unwrap();
        child.wait().unwrap();
        let mut buf = String::new();
        child.stdout.unwrap().read_to_string(&mut buf).unwrap();
        buf
    }
    use super::*;

    fn equal_tokens(expected: &TokenStream, actual: &TokenStream) {
        assert_eq!(
            format_snippet(&expected.to_string()),
            format_snippet(&actual.to_string())
        );
    }

    #[test]
    fn first() {
        let input: Item = syn::parse_quote! {
            pub trait IsOwnable {
                /// Get current admin
                fn admin_get(&self) -> Option<Address>;
                fn admin_set(&mut self, new_admin: Address) -> Result<(), Error>;
                fn admin_set_two(&mut self, new_admin: Address);
            }
        };
        let result = generate(input);
        println!("{}", format_snippet(&result.to_string()));

        let output = quote! {
            pub trait IsOwnable {
                /// Get current admin
                fn admin_get(&self) -> Option<Address>;
                fn admin_set(&mut self, new_admin: Address) -> Result<(), Error>;
                fn admin_set_two(&mut self, new_admin: Address);
            }
            pub trait Ownable {
                /// Type that implments the instance type
                type Impl: Lazy + IsOwnable + Default;
                /// Get current admin
                fn admin_get() -> Option<Address> {
                    Self::Impl::get_lazy().unwrap_or_default().admin_get()
                }
                fn admin_set(new_admin: Address) -> Result<(), Error> {
                    let mut impl_ = Self::Impl::get_lazy().unwrap_or_default();
                    let res = impl_.admin_set(new_admin)?;
                    Self::Impl::set_lazy(impl_);
                    Ok(res)
                }
                fn admin_set_two(new_admin: Address) {
                    let mut impl_ = Self::Impl::get_lazy().unwrap_or_default();
                    let res = impl_.admin_set_two(new_admin);
                    Self::Impl::set_lazy(impl_);
                    res
                }
            }

        };
        equal_tokens(&output, &result);
        // let impl_ = syn::parse_str::<ItemImpl>(result.as_str()).unwrap();
        // println!("{impl_:#?}");
    }

    #[test]
    fn second() {
        let input: Item = syn::parse_quote! {
            pub trait IsSubcontract {
                /// Get current admin
                fn riff_get(&self) -> Option<String>;
                fn riff_set(&mut self, new_riff: Address) -> Result<(), Error>;
                fn riff_set_two(&mut self, new_riff: Address);
            }
        };
        let result = generate(input);
        println!("{}", format_snippet(&result.to_string()));

        let output = quote! {
            pub trait IsSubcontract {
                /// Get current admin
                fn riff_get(&self) -> Option<String>;
                fn riff_set(&mut self, new_riff: Address) -> Result<(), Error>;
                fn riff_set_two(&mut self, new_riff: Address);
            }
            pub trait Subcontract {
                /// Type that implments the instance type
                type Impl: Lazy + IsSubcontract + Default;
                /// Get current admin
                fn riff_get() -> Option<String> {
                    Self::Impl::get_lazy().unwrap_or_default().riff_get()
                }
                fn riff_set(new_riff: Address) -> Result<(), Error> {
                    let mut impl_ = Self::Impl::get_lazy().unwrap_or_default();
                    let res = impl_.riff_set(new_riff)?;
                    Self::Impl::set_lazy(impl_);
                    Ok(res)
                }
                fn riff_set_two(new_riff: Address) {
                    let mut impl_ = Self::Impl::get_lazy().unwrap_or_default();
                    let res = impl_.riff_set_two(new_riff);
                    Self::Impl::set_lazy(impl_);
                    res
                }
            }

        };
        equal_tokens(&output, &result);
        // let impl_ = syn::parse_str::<ItemImpl>(result.as_str()).unwrap();
        // println!("{impl_:#?}");
    }
    fn p_e(e: std::io::Error) -> std::io::Error {
        eprintln!("{e:#?}");
        e
    }
}
