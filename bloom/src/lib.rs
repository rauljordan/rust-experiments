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
    (num % m) as usize
}

struct BloomFilter {
    pub bits: Vec<u8>,
    hash_fns: Vec<Box<dyn Fn(&str) -> usize>>,
}

impl BloomFilter {
    pub fn new(num_items: usize, num_hash_fns: usize) -> Self {
        assert!(num_hash_fns > 0 && num_hash_fns < num_items);
        let mut hash_fns: Vec<Box<dyn Fn(&str) -> usize>> = vec![];
        for i in 0..=num_hash_fns {
            let f = Box::new(move |elem: &str| hash_bloom(&elem, i as u64, num_items as u64));
            hash_fns.push(f);
        }
        let mut size = num_items / 8;
        if num_items % 8 > 0 {
            size += 1;
        }
        Self {
            bits: iter::repeat(0).take(size).collect(),
            hash_fns,
        }
    }
    pub fn insert(&mut self, elem: String) {
        let indices: Vec<usize> = self.hash_fns.iter().map(|f| f(elem.as_str())).collect();
        for idx in indices.into_iter() {
            let pos = idx / 8;
            let pos_within_bits = idx % 8;
            match self.bits.get_mut(pos) {
                Some(b) => {
                    *b |= 1 << pos_within_bits;
                }
                None => panic!("index did not exist"),
            }
        }
    }
    pub fn has(&self, elem: String) -> bool {
        for f in self.hash_fns.iter() {
            let idx = f(elem.as_str());
            let pos = idx / 8;
            let pos_within_bits = idx % 8;
            match self.bits.get(pos) {
                Some(b) => {
                    let bit = (*b >> pos_within_bits) & 1;
                    if bit == 0 {
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
        let mut bf = BloomFilter::new(41, 3);
        assert_eq!(bf.bits.len(), 6);
        bf.insert("hello".to_string());
        println!("{}", bf.has("hello".to_string()));
        assert_eq!(false, bf.has("world".to_string()));
    }
}
