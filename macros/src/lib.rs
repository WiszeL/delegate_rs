use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, token::Eq, Ident, ItemStruct, LitStr, Fields, Field,
};

/// A custom parser that expects an attribute of the form: error = "Exception"
struct DelegateAttr {
    error: LitStr,
}

impl Parse for DelegateAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Expect the identifier "error"
        let ident: Ident = input.parse()?;
        if ident != "error" {
            return Err(input.error("expected `error`"));
        }

        // Parse the '=' token
        input.parse::<Eq>()?;
        
        // Parse a literal string (e.g., "Exception")
        let lit: LitStr = input.parse()?;
        Ok(DelegateAttr { error: lit })
    }
}

/// The attribute macro.
/// Usage example:
///   #[delegate(error = "Exception")]
///   struct ConsumerTest { ... }
#[proc_macro_attribute]
pub fn delegate(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute using our custom parser.
    let delegate_attr = parse_macro_input!(attr as DelegateAttr);
    // Get the error type as a string.
    let error_type = delegate_attr.error.value();
    // Create an identifier from the error type.
    let error_ident = Ident::new(&error_type, proc_macro2::Span::call_site());

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
                ty: syn::parse_quote!(std::sync::Arc<DelegateManager<#error_ident>>),
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
            pub fn get_delegate_manager(&self) -> &std::sync::Arc<DelegateManager<#error_ident>> {
                &self.delegate_manager
            }
        }
    };

    TokenStream::from(expanded)
}
