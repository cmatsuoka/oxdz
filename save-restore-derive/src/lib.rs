#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(SaveRestore)]
pub fn save_restore(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();
    
    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_save_restore(&ast);
    
    // Return the generated impl
    gen.parse().unwrap()
}

fn impl_save_restore(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        use std::ptr;
        use std::mem;

        impl SaveRestore for #name {
            unsafe fn save(&self) -> Vec<u8> {
                let size = mem::size_of::<Self>();
                let mut dst: Vec<u8> = Vec::with_capacity(size);
                dst.set_len(size);
                ptr::copy(self, mem::transmute(dst.as_mut_ptr()), 1);
                dst
            }

            unsafe fn restore(&mut self, buffer: &Vec<u8>) {
                let size = buffer.len();
                ptr::copy(buffer.as_ptr(), mem::transmute(self), size)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
