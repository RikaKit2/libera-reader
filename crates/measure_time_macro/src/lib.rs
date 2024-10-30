//noinspection ALL,RsUnresolvedPath
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

//noinspection ALL
#[proc_macro_attribute]
pub fn measure_time(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // Parse the input tokens into a syntax tree
  let input = parse_macro_input!(item as ItemFn);

  // Get the function name, arguments, return type, and block
  let fn_name = &input.sig.ident;
  let fn_args = &input.sig.inputs;
  let return_type = &input.sig.output;
  let block = &input.block;

  // Check if the function is async
  let is_async = input.sig.asyncness.is_some();

  // Generate the new function with timing
  let gen = if is_async {
    quote! {
            async fn #fn_name(#fn_args) #return_type {
                let start = std::time::Instant::now();
                let result = {
                    let fut = async { #block }; // Capture the future
                    fut.await // Await the future
                };
                let duration = start.elapsed();
                event!(Level::INFO, "Function {:?} executed in: {:?}", stringify!(#fn_name), duration);
                result // Return the result
            }
        }
  } else {
    quote! {
            fn #fn_name(#fn_args) #return_type {
                let start = std::time::Instant::now();
                let result = (|| #block)(); // Capture the result of the block
                let duration = start.elapsed();
                event!(Level::INFO, "Function {:?} executed in: {:?}", stringify!(#fn_name), duration);
                result // Return the result
            }
        }
  };

  // Convert the generated code back to TokenStream
  gen.into()
}
