use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;
use syn::*;

mod generator;

#[proc_macro_attribute]
pub fn service(_: TokenStream, item: TokenStream) -> TokenStream {
    let t = syn::parse::<ItemTrait>(item).unwrap();
    let svc = parse_service(&t);
    let code = generator::generate(svc);
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
                            let x = &p.ty;
                            quote!(#x).to_string()
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
                ReturnType::Type(_, x) => {
                    output_ty = quote!(#x).to_string();
                }
                _ => unreachable!(),
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
