use std::sync::RwLock;
use rand::Rng;
use crate::utils::types::*;

static OBJECT_IDS: RwLockVec<ObjectId> = RwLock::new(Vec::new());

pub fn generate_object_id() -> ObjectId {
	let mut id: ObjectId;
	loop {
		id = rand::thread_rng().gen();
		if !OBJECT_IDS.read().unwrap().contains(&id) {
			break;
		}
	}

	OBJECT_IDS.write().unwrap().push(id);
	id
}

pub fn free_object_id(id: ObjectId) {
	let mut list = OBJECT_IDS.write().unwrap();
	let index = list.iter().position(|&x| x == id);
	if let Some(idx) = index {
		list.remove(idx);
	}
}

static GENERIC_IDS: RwLockVec<GenericId> = RwLock::new(Vec::new());

pub fn generate_generic_id() -> GenericId {
	let mut id: GenericId;
	loop {
		id = rand::thread_rng().gen();
		if !GENERIC_IDS.read().unwrap().contains(&id) {
			break;
		}
	}

	GENERIC_IDS.write().unwrap().push(id);
	id
}

pub fn free_generic_id(id: GenericId) {
	let mut list = GENERIC_IDS.write().unwrap();
	let index = list.iter().position(|&x| x == id);
	if let Some(idx) = index {
		list.remove(idx);
	}
}
