use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ByteParser)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let DeriveInput { ident, .. } = input;

    let output = quote! {
        // pub fn default() -> #ident {
        //     #ident::default()
        // }
        impl ByteCode for #ident {}
        impl ByteRead for #ident {}
        impl ByteParse for #ident {
            fn offset(&self) -> usize {
                self.offset
            }
            fn length(&self) -> usize {
                self.byte_count as usize
            }
            fn get(&self, offset: usize) -> Option<&u8> {
                self.raw.get(offset)
            }
            fn skip(&mut self, num: u32) {
                self.offset += num as usize
            }
        }
    };

    output.into()
}
