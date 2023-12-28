use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn time_execution(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(item as ItemFn);
    let log_message = format!("EXECUTION_TIME({}): {{}}", sig.ident);
    let stmts = &block.stmts;
    let out = quote! {
        #(#attrs)* #vis #sig {
            let time_execution_start = Instant::now();
            let out = {
                #(#stmts)*
            };
            let time_execution_elapsed = time_execution_start.elapsed();
            log::info!(#log_message, time_execution_elapsed.as_micros());
            out
        }
    };
    proc_macro::TokenStream::from(out)
}

