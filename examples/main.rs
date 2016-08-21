extern crate bloomfilter;

use bloomfilter::BloomFilter;

fn main() {
    let mut b = BloomFilter::new(30);
    b.add("frog");
    let e = b.exists("dog");
    println!("{}", e)
}
