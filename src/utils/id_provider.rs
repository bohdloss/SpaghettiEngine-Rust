use std::sync::RwLock;
use rand::Rng;
use crate::utils::types::*;

static IDS: RwLockVec<u64> = RwLock::new(Vec::new());

pub fn generate_id() -> u64 {
	let mut id: u64;
	loop {
		id = rand::thread_rng().gen();
		if !IDS.read().unwrap().contains(&id) {
			break;
		}
	}

	IDS.write().unwrap().push(id);
	id
}

pub fn free_id(id: u64) {
	let mut list = IDS.write().unwrap();
	let index = list.iter().position(|&x| x == id);
	if let Some(idx) = index {
		list.remove(idx);
	}
}
