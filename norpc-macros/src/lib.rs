use proc_macro::TokenStream;
use proc_macro2::*;
use quote::quote;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream, Result};
use syn::*;

mod generator;

struct Args {
    local: bool,
}

mod kw {
    syn::custom_keyword!(Send);
}

fn try_parse(input: ParseStream) -> Result<Args> {
    if input.peek(Token![?]) {
        input.parse::<Token![?]>()?;
        input.parse::<kw::Send>()?;
        Ok(Args { local: true })
    } else {
        Ok(Args { local: false })
    }
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let args: Args = try_parse(input)?;
        Ok(args)
    }
}

#[proc_macro_attribute]
pub fn service(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let t = syn::parse::<ItemTrait>(item).unwrap();
    let svc = parse_service(&t);
    let generator = generator::Generator {
        no_send: args.local,
    };
    let code = generator.generate(svc);
    TokenStream::from_str(&code).unwrap()
}

#[derive(Debug)]
struct Service {
    name: String,
    functions: Vec<Function>,
}
#[derive(Debug)]
struct Function {
    name: String,
    inputs: Vec<Parameter>,
    output: String,
}
#[derive(Debug)]
struct Parameter {
    var_name: String,
    typ_name: String,
}

fn parse_service(t: &ItemTrait) -> Service {
    let svc_name = {
        let x = &t.ident;
        quote!(#x).to_string()
    };
    let mut functions = vec![];
    for f in &t.items {
        functions.push(parse_func(f));
    }
    Service {
        name: svc_name,
        functions,
    }
}
fn parse_func(f: &TraitItem) -> Function {
    match f {
        TraitItem::Method(m) => {
            let sig = &m.sig;

            let x = &sig.ident;
            let func_name = quote!(#x).to_string();

            let mut inputs = vec![];
            for input in &sig.inputs {
                match input {
                    FnArg::Typed(p) => {
                        let var_name = {
                            let x = &p.pat;
                            quote!(#x).to_string()
                        };
                        let var_type = {
                            let ty = &p.ty;
                            quote!(#ty).to_string()
                        };
                        inputs.push(Parameter {
                            var_name,
                            typ_name: var_type,
                        });
                    }
                    _ => unreachable!(),
                }
            }

            let output_ty;
            match &sig.output {
                ReturnType::Type(_, ty) => {
                    output_ty = quote!(#ty).to_string();
                }
                ReturnType::Default => {
                    output_ty = "()".to_string();
                }
            }
            Function {
                name: func_name,
                inputs,
                output: output_ty,
            }
        }
        // TODO ignore here to skip comments
        _ => unreachable!(),
    }
}