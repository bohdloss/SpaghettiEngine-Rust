use crate::utils::id_type::id_type;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use std::{mem, thread};

struct Void {
    _ptr: *const c_void,
}

unsafe impl Send for Void {}
unsafe impl Sync for Void {}

static TASK_LIST: Lazy<Mutex<HashMap<TaskHandle, Box<dyn FnMut() + Sync + Send>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static EVENT_LIST: Lazy<Mutex<HashMap<EventHandle, Box<dyn FnOnce() -> Void + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static EVENT_RETURN: Lazy<Mutex<HashMap<EventHandle, Void>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static SHUTDOWN_EVENT_LIST: Lazy<Mutex<Vec<Box<dyn FnOnce() + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

static SLEEP_DURATION: Duration = Duration::from_millis(1);

id_type!(TaskHandle);
id_type!(EventHandle);

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

pub fn send_event<T, R>(event: T) -> R
where
    T: FnOnce() -> R + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    let id = EventHandle::new();
    EVENT_LIST.lock().unwrap().insert(
        id,
        Box::new(|| unsafe { mem::transmute(Box::new(event())) }),
    );

    while !EVENT_RETURN.lock().unwrap().contains_key(&id) {
        thread::sleep(SLEEP_DURATION);
    }

    let generic = EVENT_RETURN.lock().unwrap().remove(&id).unwrap();

    // Since the request id is unique for each request,
    // we know the request will take the result of event()
    // (which is of type R) and wrap it in a Box,
    // therefore the result can't be anything but a Box<R>.
    // Therefore, it is safe to transmute the Box without
    // any runtime checks, and dereference its value to return it
    unsafe {
        let result: Box<R> = mem::transmute(generic);
        *result
    }
}

pub fn register_shutdown_event<T>(f: T)
where
    T: FnOnce() + Send + Sync + 'static,
{
    SHUTDOWN_EVENT_LIST.lock().unwrap().push(Box::new(f));
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
    shutdown();
    thread.join().unwrap();
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
    shutdown();
    thread.join().unwrap();
}

fn do_loop_body() {
    thread::sleep(SLEEP_DURATION);

    {
        let mut events = EVENT_LIST.lock().unwrap();
        while events.len() > 0 {
            let mut handle: Option<EventHandle> = None;
            for (id, _) in events.iter() {
                handle = Some(*id);
                break;
            }

            match handle {
                Some(id) => match events.remove(&id) {
                    Some(event) => {
                        let result = event();
                        EVENT_RETURN.lock().unwrap().insert(id, result);
                    }
                    None => {}
                },
                None => {}
            }
        }
    }

    {
        let mut tasks = TASK_LIST.lock().unwrap();
        for (_, task) in tasks.iter_mut() {
            task();
        }
    }
}

fn shutdown() {
    let mut events = SHUTDOWN_EVENT_LIST.lock().unwrap();
    while events.len() > 0 {
        let mut idx: Option<usize> = None;
        for (i, _) in events.iter().enumerate() {
            idx = Some(i);
            break;
        }

        match idx {
            Some(id) => {
                let event = events.remove(id);
                event();
            }
            None => {}
        }
    }

    TASK_LIST.lock().unwrap().clear();
    EVENT_LIST.lock().unwrap().clear();
    EVENT_RETURN.lock().unwrap().clear();
}
