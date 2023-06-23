use crate::utils::id_type::id_type;
use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static TASK_LIST: Lazy<Mutex<HashMap<TaskHandle, Box<dyn FnMut() + Sync + Send + 'static>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static EVENT_LIST: Lazy<Mutex<VecDeque<Box<dyn FnOnce() + Sync + Send + 'static>>>> =
    Lazy::new(|| Mutex::new(VecDeque::new()));

static SLEEP_DURATION: Duration = Duration::from_millis(50);

id_type!(TaskHandle);

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
    T: FnOnce() + Sync + Send + 'static,
{
    let mut list = EVENT_LIST.lock().unwrap();
    list.push_back(Box::new(f));
}

#[macro_export]
macro_rules! spaghetti_entry_point {
    ($function:ident($($arg:expr),*)) => {{
        use $crate::core::entry_point;
        entry_point::main_thread_entry_point(move || {
            $function($($arg),*);
        });
    }};
}

#[macro_export]
macro_rules! spaghetti_debug_entry_point {
    ($thread_body:expr, $loop_condition:expr) => {{
        use $crate::core::entry_point;
        entry_point::debug_entry_point($thread_body, $loop_condition);
    }};
    ($thread_body:expr) => {{
        use $crate::core::entry_point;
        entry_point::debug_entry_point($thread_body, |val| val);
    }};
}

pub fn main_thread_entry_point<T>(thread_body: T)
where
    T: Fn() + Send + 'static,
{
    let thread = thread::spawn(move || {
        thread_body();
    });

    while !thread.is_finished() {
        do_loop_body();
    }
    reset();
}

pub fn debug_entry_point<T, F>(thread_body: T, loop_condition: F)
where
    T: Fn() + Send + 'static,
    F: Fn(bool) -> bool,
{
    let thread = thread::spawn(move || {
        thread_body();
    });

    while loop_condition(!thread.is_finished()) {
        do_loop_body();
    }
    reset();
}

fn do_loop_body() {
    thread::sleep(SLEEP_DURATION);

    {
        let mut events = EVENT_LIST.lock().unwrap();
        while let Some(event) = events.pop_front() {
            event();
        }
    }

    {
        let mut tasks = TASK_LIST.lock().unwrap();
        for (_, task) in tasks.iter_mut() {
            task();
        }
    }
}

fn reset() {
    TASK_LIST.lock().unwrap().clear();
    EVENT_LIST.lock().unwrap().clear();
}
