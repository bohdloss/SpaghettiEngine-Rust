use std::sync::Mutex;
use std::time::SystemTime;
use crate::log;

struct Number {
    a: i32,
    b: i32,
    c: i32
}

fn mutex_test() {
    let mutex = Mutex::new(Number {a: 0, b: 3, c: 9});

    let time = SystemTime::now();
    let mut times: usize = 0;
    let seconds = 10;
    while SystemTime::now().duration_since(time).unwrap().as_millis() < 1000 * seconds {
        let mut data = mutex.lock().unwrap();

        data.b = data.a + data.c;
        data.c = data.a * data.b;

        times += 1;
    }
    log!(Info, "Iterations: {}, Seconds: {}, Average: {} ms", times, seconds, (seconds as f64 / times as f64));
}