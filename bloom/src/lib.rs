use sha3::{Digest, Sha3_256};
use std::{io::Read, iter};

fn hash_bloom(elem: &str, i: u64, m: u64) -> usize {
    assert!(i < 32);
    let mut hasher = Sha3_256::new();
    hasher.update(elem);
    let result = hasher.finalize();
    let mut buf = [0; 8];
    let mut handle = result.take(8);
    handle.read(&mut buf).unwrap();
    let num: u64 = u64::from_be_bytes(buf).checked_add(i).unwrap();
    println!("{} and {}", num, num % m);
    (num % m) as usize
}

struct BloomFilter {
    // TODO: Bad. a bool is 1 byte and we want 1 bit for max-efficiency.
    pub bits: Vec<bool>,
    hash_fns: Vec<Box<dyn Fn(&str) -> usize>>,
}

impl BloomFilter {
    pub fn new(m: usize, k: usize) -> Self {
        assert!(k > 0 && k < m);
        let mut hash_fns: Vec<Box<dyn Fn(&str) -> usize>> = vec![];
        for i in 0..=k {
            let f = Box::new(move |elem: &str| hash_bloom(&elem, i as u64, m as u64));
            hash_fns.push(f);
        }
        Self {
            bits: iter::repeat(false).take(m).collect(),
            hash_fns,
        }
    }
    pub fn insert(&mut self, elem: String) {
        let indices: Vec<usize> = self.hash_fns.iter().map(|f| f(elem.as_str())).collect();
        for idx in indices.into_iter() {
            match self.bits.get_mut(idx) {
                Some(b) => *b = true,
                None => panic!("index did not exist"),
            }
        }
    }
    pub fn has(&self, elem: String) -> bool {
        for f in self.hash_fns.iter() {
            let idx = f(elem.as_str());
            match self.bits.get(idx) {
                Some(b) => {
                    if !b {
                        return false;
                    }
                }
                None => panic!("index did not exist"),
            }
        }
        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut bf = BloomFilter::new(10, 3);
        assert_eq!(bf.bits.len(), 10);
        bf.insert("hello".to_string());
        println!("{}", bf.has("hello".to_string()));
        println!("{}", bf.has("world".to_string()));
    }
}
