pub fn bubble_sort<T: Ord>(items: &mut [T]) {
    for i in 0..items.len() {
        for j in 0..items.len() - 1 - i {
            if items[j] > items[j + 1] {
                items.swap(j, j + 1);
            }
        }
    }
}

fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

fn main() {
    let mut items = vec![2, 4, 2, 0, 1];
    let it = rdtsc();
    bubble_sort(items.as_mut_slice());
    println!("{}", rdtsc() - it);
}
