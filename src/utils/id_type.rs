#[macro_export]
macro_rules! id_type {
	($name:ident) => {
		pub struct $name {
			id: u64
		}

		impl $name {
			#[allow(unused)]
			fn new() -> Self {
				Self {
					id: crate::utils::id_provider::generate_id()
				}
			}

			#[allow(unused)]
			fn from(id: u64) -> Self {
				Self {
					id
				}
			}
		}

		impl Copy for $name {
		}

		impl Eq for $name {
		}

		impl std::hash::Hash for $name {
			fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
				state.write_u64(self.id);
				state.finish();
			}

			fn hash_slice<H: std::hash::Hasher>(data: &[Self], state: &mut H) where Self: Sized {
				for val in data {
					state.write_u64(val.id);
				}
				state.finish();
			}
		}

		impl Clone for $name {
			fn clone(&self) -> Self {
				Self {
					id: self.id
				}
			}
		}

		impl PartialEq for $name {
			fn eq(&self, other: &Self) -> bool {
				self.id == other.id
			}
		}
	}
}

pub use id_type;