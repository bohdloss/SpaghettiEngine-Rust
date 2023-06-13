use crate::utils::id_type::id_type;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use std::{mem, thread};

static TASK_LIST: Lazy<Mutex<HashMap<TaskHandle, Box<dyn FnMut() + Sync + Send + 'static>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static EVENT_LIST: Lazy<Mutex<Vec<Box<dyn FnMut() + Sync + Send + 'static>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
static STOP_SIGNAL: Mutex<bool> = Mutex::new(false);
static SLEEP_DURATION: Duration = Duration::from_millis(50);

id_type!(TaskHandle);

pub fn stop_signal() {
    let mut value = STOP_SIGNAL.lock().unwrap();
    mem::replace(&mut *value, true);
}

pub fn register_task<T>(f: T) -> TaskHandle
where
    T: FnMut() + Sync + Send + 'static,
{
    let mut list = TASK_LIST.lock().unwrap();
    let handle = TaskHandle::new();
    list.insert(handle, Box::new(f));
    handle
}

pub fn unregister_task(handle: TaskHandle) {
    let mut list = TASK_LIST.lock().unwrap();
    list.remove(&handle);
}

pub fn send_event<T>(f: T)
where
    T: FnMut() + Sync + Send + 'static,
{
    let mut list = EVENT_LIST.lock().unwrap();
    list.push(Box::new(f));
}

#[macro_export]
macro_rules! spaghetti_entry_point {
    ($function:ident) => {{
        std::thread::spawn(|| {
            $function();
        });
        $crate::core::entry_point::main_thread_entry_point();
    }};
}

pub fn main_thread_entry_point() {
    while !*STOP_SIGNAL.lock().unwrap() {
        thread::sleep(SLEEP_DURATION);

        {
            let mut events = EVENT_LIST.lock().unwrap();
            for event in events.iter_mut() {
                event();
            }
            events.clear();
        }

        {
            let mut tasks = TASK_LIST.lock().unwrap();
            for (handle, task) in tasks.iter_mut() {
                task();
            }
        }
    }
}
