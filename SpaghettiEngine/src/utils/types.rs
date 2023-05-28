use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::{rc, sync};
use std::rc::{Rc};
use std::sync::{Arc, Mutex, RwLock};
use cgmath::{Vector2, Vector3, Vector4};

/*
Used only to type less pointy brackets
 */
pub type RcCell<T> = Rc<Cell<T>>;
pub type RcRefCell<T> = Rc<RefCell<T>>;

pub type ArcRwLock<T> = Arc<RwLock<T>>;
pub type ArcMutex<T> = Arc<Mutex<T>>;

pub type WeakCell<T> = rc::Weak<Cell<T>>;
pub type WeakRefCell<T> = rc::Weak<RefCell<T>>;

pub type WeakRwLock<T> = sync::Weak<RwLock<T>>;
pub type WeakMutex<T> = sync::Weak<Mutex<T>>;

pub type CellHashMap<K, V> = Cell<HashMap<K, V>>;
pub type RefCellHashMap<K, V> = RefCell<HashMap<K, V>>;
pub type RcCellHashMap<K, V> = RcCell<HashMap<K, V>>;
pub type RcRefCellHashMap<K, V> = RcRefCell<HashMap<K, V>>;
pub type WeakCellHashMap<K, V> = WeakCell<HashMap<K, V>>;
pub type WeakRefCellHashMap<K, V> = WeakRefCell<HashMap<K, V>>;

pub type BoxHashMap<K, V> = Box<HashMap<K, V>>;

pub type RwLockHashMap<K, V> = RwLock<HashMap<K, V>>;
pub type MutexHashMap<K, V> = Mutex<HashMap<K, V>>;
pub type ArcRwLockHashMap<K, V> = ArcRwLock<HashMap<K, V>>;
pub type ArcMutexHashMap<K, V> = ArcMutex<HashMap<K, V>>;
pub type WeakRwLockHashMap<K, V> = WeakRwLock<HashMap<K, V>>;
pub type WeakMutexHashMap<K, V> = WeakMutex<HashMap<K, V>>;

pub type CellVec<T> = Cell<Vec<T>>;
pub type RefCellVec<T> = RefCell<Vec<T>>;
pub type RcCellVec<T> = RcCell<Vec<T>>;
pub type RcRefCellVec<T> = RcRefCell<Vec<T>>;
pub type WeakCellVec<T> = WeakCell<Vec<T>>;
pub type WeakRefCellVec<T> = WeakRefCell<Vec<T>>;

pub type BoxVec<T> = Box<Vec<T>>;

pub type RwLockVec<T> = RwLock<Vec<T>>;
pub type MutexVec<T> = Mutex<Vec<T>>;
pub type ArcRwLockVec<T> = ArcRwLock<Vec<T>>;
pub type ArcMutexVec<T> = ArcMutex<Vec<T>>;
pub type WeakRwLockVec<T> = WeakRwLock<Vec<T>>;
pub type WeakMutexVec<T> = WeakMutex<Vec<T>>;

pub type CellVecDeque<T> = Cell<VecDeque<T>>;
pub type RefCellVecDeque<T> = RefCell<VecDeque<T>>;
pub type RcCellVecDeque<T> = RcCell<VecDeque<T>>;
pub type RcRefCellVecDeque<T> = RcRefCell<VecDeque<T>>;
pub type WeakCellVecDeque<T> = WeakCell<VecDeque<T>>;
pub type WeakRefCellVecDeque<T> = WeakRefCell<VecDeque<T>>;

pub type BoxVecDeque<T> = Box<VecDeque<T>>;

pub type RwLockVecDeque<T> = RwLock<VecDeque<T>>;
pub type MutexVecDeque<T> = Mutex<VecDeque<T>>;
pub type ArcRwLockVecDeque<T> = ArcRwLock<VecDeque<T>>;
pub type ArcMutexVecDeque<T> = ArcMutex<VecDeque<T>>;
pub type WeakRwLockVecDeque<T> = WeakRwLock<VecDeque<T>>;
pub type WeakMutexVecDeque<T> = WeakMutex<VecDeque<T>>;

// Vectors

// Change this to change the precision between 32 / 64 bits for the whole engine
#[allow(non_camel_case_types)]
pub type float = f32;

pub type Vector2i = Vector2<i32>;
pub type Vector2f = Vector2<float>;

pub type Vector3i = Vector3<i32>;
pub type Vector3f = Vector3<float>;

pub type Vector4i = Vector4<i32>;
pub type Vector4f = Vector4<float>;