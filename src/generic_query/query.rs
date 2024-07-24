use proc_macro::TokenStream;

use super::QueryObject;

pub trait QueryExecutor {
    fn execute_query(queries: Vec<QueryObject>) -> Vec<Self> where Self: Sized;
}

fn impl_query_executor_macro(ast: &syn::DeriveInput) -> TokenStream {
    let object_name = &ast.ident;
    let gen = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };
    gen.into()
}