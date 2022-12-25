use rand::prelude::*;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

fn main() -> io::Result<()> {
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

    let mut rng = thread_rng();

    loop {
        let now = std::time::Instant::now();
        fuzz_entries(&rng, &entries)?;
        let elapsed = now.elapsed().as_secs_f64();
        println!("fcps {:?}", (entries.len() as f64) / elapsed);
    }
}

fn fuzz_entries(rng: &ThreadRng, entries: &Vec<Vec<u8>>) -> io::Result<()> {
    for entry in entries.iter() {
        let num_elems_to_mutate = rng.gen_range(0..entry.len());
        println!("{}", num_elems_to_mutate);
        let input = entry.as_slice();
        fuzz(input)?;
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
            io::stderr().write_all(&output.stderr).unwrap();
        }
        Err(e) => {
            println!("Failed with {:?}", e);
        }
    }
    Ok(())
}
