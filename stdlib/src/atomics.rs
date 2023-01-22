/// what to build? Build a debugger!
pub struct WasmDebugger {}

impl WasmDebugger {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod test {
    use std::thread;
    use std::time::Duration;
    use std::{collections::VecDeque, sync::Condvar, sync::Mutex};
    fn parking() {
        let queue = Mutex::new(VecDeque::new());

        thread::scope(|s| {
            let t = s.spawn(|| loop {
                let item = queue.lock().unwrap().pop_front();
                if let Some(item) = item {
                    dbg!(item);
                } else {
                    thread::park();
                }
            });

            for i in 0.. {
                queue.lock().unwrap().push_back(i);
                t.thread().unpark();
                thread::sleep(Duration::from_secs(1));
            }
        })
    }
    #[test]
    fn condvar() {
        let queue = Mutex::new(VecDeque::new());
        let not_empty = Condvar::new();

        thread::scope(|s| {
            let t = s.spawn(|| loop {
                let mut q = queue.lock().unwrap();
                let item = loop {
                    if let Some(item) = q.pop_front() {
                        break item;
                    } else {
                        q = not_empty.wait(q).unwrap();
                    }
                };
                drop(q);
                dbg!(item);
            });

            for i in 0.. {
                queue.lock().unwrap().push_back(i);
                not_empty.notify_one();
                thread::sleep(Duration::from_secs(1));
            }
        })
    }
}
