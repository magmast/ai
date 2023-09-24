use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, ExprAssign, ExprLit, Lit};

#[proc_macro_derive(FunctionArguments, attributes(arg))]
pub fn derive_function_arguments(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;

    let properties = if let Data::Struct(data_struct) = input.data {
        data_struct
            .fields
            .into_iter()
            .fold(quote! {}, |acc, field| {
                let name = field.ident.map(|ident| ident.to_string());
                let description = field
                    .attrs
                    .into_iter()
                    .find(|arg| arg.path().is_ident("arg"))
                    .map(|arg| {
                        arg.parse_args::<ExprAssign>()
                            .expect("Expected to find assignment expression.")
                    })
                    .and_then(|expr| match (*expr.left, *expr.right) {
                        (
                            Expr::Path(path),
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit), ..
                            }),
                        ) if path.path.is_ident("description") => {
                            Some(quote! { "description": #lit, })
                        }
                        _ => None,
                    });

                quote! {
                    #acc
                    #name: {
                        "type": "string",
                        #description
                    },
                }
            })
    } else {
        todo!("Only data struct are supported currently.")
    };

    let expanded = quote! {
        impl FunctionArguments for #ident {
            fn json_schema() -> ::serde_json::Value {
                ::serde_json::json!({
                    "type": "object",
                    "properties": {
                        #properties
                    }
                })
            }
        }
    };

    TokenStream::from(expanded)
}
