use std::sync::{Arc, Mutex, RwLock};
use array_init::array_init;
use crate::utils::types::*;

struct ObjectEntry<T> {
	object: ArcRwLock<T>,
	in_use: bool
}

pub struct ObjectPool<T, const POOL_SIZE: usize> {
	mutex: Mutex<()>,
	objects: [ObjectEntry<T>; POOL_SIZE],
	get_pointer: usize,
	drop_pointer: usize
}

impl<T, const POOL_SIZE: usize> ObjectPool<T, POOL_SIZE> {

	pub fn new(constructor: fn() -> T) -> Self {
		ObjectPool {
			mutex: Mutex::new(()),
			objects: array_init(|_| {
				ObjectEntry {
					object: Arc::new(RwLock::new(constructor())),
					in_use: false
				}
			}),
			get_pointer: 0,
			drop_pointer: 0,
		}
	}

	pub fn borrow(&mut self) -> ArcRwLock<T> {
		let mut cycles: usize = 0;
		loop {
			let entry = &mut self.objects[self.get_pointer];
			if self.get_pointer == POOL_SIZE - 1 {
				self.get_pointer = 0;
			} else {
				self.get_pointer += 1;
			}

			if entry.in_use {
				if let Ok(_) = self.mutex.lock() {
					if entry.in_use {
						entry.in_use = true;
						return entry.object.clone();
					}
				} else {
					panic!();
				}
			}

			cycles += 1;
			if cycles >= POOL_SIZE * 2 {
				panic!();
			}
		}
	}

	pub fn pay(&mut self, object: ArcRwLock<T>) {
		let mut cycles: usize = 0;
		loop {
			let entry = &mut self.objects[self.drop_pointer];
			if self.drop_pointer == 0{
				self.drop_pointer = POOL_SIZE - 1;
			} else {
				self.drop_pointer -= 1;
			}

			if Arc::ptr_eq(&entry.object, &object) {
				if let Ok(_) = self.mutex.lock() {
					entry.in_use = false;
					break;
				} else {
					panic!();
				}
			}

			cycles += 1;
			if cycles >= POOL_SIZE * 2 {
				break;
			}
		}
	}

}
