use std::sync::RwLock;
use once_cell::sync::Lazy;
use rand::Rng;
use crate::utils::types::*;

static mut IDS: Lazy<RwLockVec<IdType>> = Lazy::new(|| RwLock::new(Vec::new()));

pub fn generate_id() -> IdType {
	let mut id: IdType;
	loop {
		id = rand::thread_rng().gen();
		if let Ok(list) = unsafe{&IDS}.read() {
			if !list.contains(&id) {
				break;
			}
		} else {
			panic!();
		}
	}

	if let Ok(mut list) = unsafe{&IDS}.write() {
		list.push(id);
	} else {
		panic!();
	}
	id
}

pub fn free_id(id: IdType) {
	if let Ok(mut list) = unsafe{&IDS}.write() {
		let index = list.iter().position(|&x| x == id);
		if let Some(idx) = index {
			list.remove(idx);
		}
	} else {
		panic!();
	}
}