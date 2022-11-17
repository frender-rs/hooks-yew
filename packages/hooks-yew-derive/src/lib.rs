use darling::FromMeta;
use proc_macro::TokenStream;

mod macro_imp;

#[proc_macro_attribute]
pub fn hook_component(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args as syn::AttributeArgs);

    let args = match macro_imp::HookComponentArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let item_fn = syn::parse_macro_input!(input as syn::ItemFn);

    let (mut ts, error) = args.transform_item_fn(item_fn);

    if let Some(error) = error {
        ts.extend(error.write_errors());
    }

    ts.into()
}
