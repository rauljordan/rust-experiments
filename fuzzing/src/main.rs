use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

mod riscv;

use riscv::*;

struct Tracker {
    cases_processed: AtomicI32,
}

impl Tracker {
    pub fn new() -> Self {
        Self {
            cases_processed: AtomicI32::new(0),
        }
    }
}

fn main() {
    let mut emu = Emulator::new(1024 * 1024);
    let tmp = emu.mmu.allocate(4).unwrap();
    emu.mmu.write_from(tmp, b"asdf").unwrap();
    {
        let mut forked = emu.mmu.fork();
        for ii in 0..100_100_100 {
            emu.mmu.write_from(tmp, b"asdf").unwrap();
            forked.reset(&emu.mmu);
        }
    }
}

fn fuzzy_main() -> io::Result<()> {
    let wd = env::current_dir().unwrap();
    let corpus_path = wd.join("fuzzing/corpus");
    let entries: Vec<Vec<u8>> = fs::read_dir(corpus_path)
        .unwrap()
        .into_iter()
        .map(|e| {
            let path: PathBuf = e.unwrap().path();
            fs::read(path).unwrap()
        })
        .collect();

    // let mut rng = thread_rng();

    let tracker = Arc::new(Tracker::new());
    for _ in 0..8 {
        let tracker = tracker.clone();
        let entries = entries.clone();
        std::thread::spawn(move || worker(tracker, entries));
    }

    let start = std::time::Instant::now();
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let elapsed = start.elapsed().as_secs_f64();
        let cases = tracker.cases_processed.load(Ordering::SeqCst);
        println!("fcps {:?}", (cases as f64) / elapsed);
    }
}

fn worker(tracker: Arc<Tracker>, entries: Vec<Vec<u8>>) {
    loop {
        match fuzz_entries(&entries) {
            Ok(()) => {
                tracker
                    .cases_processed
                    .fetch_add(entries.len() as i32, Ordering::SeqCst);
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}

fn fuzz_entries(entries: &Vec<Vec<u8>>) -> io::Result<()> {
    for entry in entries.iter() {
        fuzz(entry)?;
    }
    Ok(())
}

/// Building a custom fuzzer:
/// Idea is that we'll assume we are fuzzing rust code
/// by providing a function with random inputs as much
/// as possible. However, to do this, we might want
/// to pick from a corpus that we know wil work
/// effectively.
fn fuzz(input: &[u8]) -> io::Result<()> {
    // Write the input to a file, and then try it in objdump.
    let fpath = "/tmp/trial";
    fs::write(&fpath, input)?;
    match Command::new("objdump").arg("-x").arg(&fpath).output() {
        Ok(output) => {
            // Ignore exit status 1, instead focus on
            // exit status 11 as a fuzz candidate.
            let code = output.status.code().unwrap();
            if code == 1 {
                return Ok(());
            }
            // io::stderr().write_all(&output.stderr).unwrap();
        }
        Err(e) => {
            println!("Failed with {:?}", e);
        }
    }
    Ok(())
}
