extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(GameEvent)]
pub fn game_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;
	let expanded = quote! {
        impl GameEvent for #name {
            fn get_event_data(&self) -> &EventData {
		        &self.event
            }
	        fn get_event_data_mut(&mut self) -> &mut EventData {
		        &mut self.event
	        }
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(AsAny)]
pub fn as_any(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = input.ident;
	let expanded = quote! {
        impl AsAny for #name {
			fn as_any(&self) -> &dyn core::any::Any {
				self
			}
        }
    };
	TokenStream::from(expanded)
}