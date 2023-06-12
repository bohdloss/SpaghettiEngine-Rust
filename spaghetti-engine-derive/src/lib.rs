extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(GameEvent, attributes(event_data))]
pub fn game_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = quote! {
        impl GameEvent for #name {
            fn get_event_data(&self) -> &EventData {
                &self.event_data
            }
            fn get_event_data_mut(&mut self) -> &mut EventData {
                &mut self.event_data
            }
            fn get_event_type(&self) -> u64 {
                let id = game_event::get_event_type::<#name>();
                if let Some(the_id) = id {
                    return the_id;
                }
                return game_event::register_event_type::<#name>(|| Box::new(#name::new_empty()));
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
