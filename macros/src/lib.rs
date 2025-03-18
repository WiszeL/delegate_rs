use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Ident, ItemStruct, Fields, Field,
};

/// The attribute macro.
/// Usage example:
///   #[delegate]
///   struct ConsumerTest { ... }
#[proc_macro_attribute]
pub fn delegate(_: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the item as a struct.
    let mut input = parse_macro_input!(item as ItemStruct);
    let struct_ident = &input.ident;

    // Ensure the struct has named fields and add a new field called `delegate_manager`.
    match &mut input.fields {
        Fields::Named(fields_named) => {
            fields_named.named.push(Field {
                mutability: syn::FieldMutability::None,
                attrs: Vec::new(),
                vis: syn::parse_quote!(pub),
                ident: Some(Ident::new("delegate_manager", proc_macro2::Span::call_site())),
                colon_token: Some(Default::default()),
                ty: syn::parse_quote!(std::sync::Arc<::delegate_rs::DelegateManager>),
            });
        }
        _ => {
            return syn::Error::new_spanned(
                &input,
                "The #[delegate] macro only supports structs with named fields.",
            )
            .to_compile_error()
            .into();
        }
    }

    // Generate an implementation block with a getter method.
    let expanded = quote! {
        #input

        impl #struct_ident {
            pub fn get_delegate_manager(&self) -> &std::sync::Arc<::delegate_rs::DelegateManager> {
                &self.delegate_manager
            }
        }
    };

    TokenStream::from(expanded)
}
