use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

/*
Used only to type less pointy brackets
 */
pub type RcCell<T> = Rc<Cell<T>>;
pub type RcRefCell<T> = Rc<RefCell<T>>;

pub type ArcRwLock<T> = Arc<RwLock<T>>;
pub type ArcMutex<T> = Arc<Mutex<T>>;

pub type CellHashMap<K, V> = Cell<HashMap<K, V>>;
pub type RefCellHashMap<K, V> = RefCell<HashMap<K, V>>;
pub type RcCellHashMap<K, V> = RcCell<HashMap<K, V>>;
pub type RcRefCellHashMap<K, V> = RcRefCell<HashMap<K, V>>;

pub type RwLockHashMap<K, V> = RwLock<HashMap<K, V>>;
pub type MutexHashMap<K, V> = Mutex<HashMap<K, V>>;
pub type ArcRwLockHashMap<K, V> = ArcRwLock<HashMap<K, V>>;
pub type ArcMutexHashMap<K, V> = ArcMutex<HashMap<K, V>>;

pub type CellVec<T> = Cell<Vec<T>>;
pub type RefCellVec<T> = RefCell<Vec<T>>;
pub type RcCellVec<T> = RcCell<Vec<T>>;
pub type RcRefCellVec<T> = RcRefCell<Vec<T>>;

pub type RwLockVec<T> = RwLock<Vec<T>>;
pub type MutexVec<T> = Mutex<Vec<T>>;
pub type ArcRwLockVec<T> = ArcRwLock<Vec<T>>;
pub type ArcMutexVec<T> = ArcMutex<Vec<T>>;