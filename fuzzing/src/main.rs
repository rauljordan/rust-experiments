/// Building a custom fuzzer:
/// Idea is that we'll assume we are fuzzing rust code
/// by providing a function with random inputs as much
/// as possible. However, to do this, we might want
/// to pick from a corpus that we know wil work
/// effectively.
fn main() {
    println!("Hello, world!");
}
