extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::__private::Span;
use syn::{DeriveInput, Ident, parse_macro_input};

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

#[proc_macro_derive(Id, attributes(bits32, bits64))]
pub fn id(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	let attributes = input.attrs;
	let mut id_type_option: Option<Ident> = None;

	for attr in attributes {
		if let Some(attr_ident) = attr.path().get_ident() {
			let attr_name = &*attr_ident.to_string();

			// Detect id type
			match attr_name {
				"bits32" => id_type_option = Some(Ident::new("u32", Span::call_site())),
				"bits64" => id_type_option = Some(Ident::new("u64", Span::call_site())),
				_ => {
					panic!("Unknown attribute: {}", attr_name);
				}
			}
		}
	}

	let id_type = id_type_option.expect("Unknown id type length");

	let original_name = &*input.ident.to_string();
	let new_name = &original_name[1..];
	let name = Ident::new(new_name, Span::call_site());

	let expanded = quote! {
		pub struct #name {
			id: #id_type
		}

		impl #name {
			fn new() -> Self {
				Self {
					id: crate::utils::id_provider::generate_id() as #id_type
				}
			}
			fn from(id: #id_type) -> Self {
				Self {
					id
				}
			}
		}

		impl Copy for #name {
		}

		impl Eq for #name {
		}

		impl std::hash::Hash for #name {
			fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
				state.write_u64(self.id as u64);
				state.finish();
			}

			fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H) where Self: Sized {
				for val in data {
					state.write_u64(val.id as u64);
				}
				state.finish();
			}
		}

		impl Clone for #name {
			fn clone(&self) -> Self {
				Self {
					id: self.id
				}
			}
		}

		impl PartialEq for #name {
			fn eq(&self, other: &Self) -> bool {
				self.id == other.id
			}
		}

    };
	TokenStream::from(expanded)
}