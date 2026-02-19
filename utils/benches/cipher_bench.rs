use iai_callgrind::{black_box, library_benchmark, library_benchmark_group, main};
use utils::cipher::{decrypt_data, encrypt_data};

// Setup function to prepare the data (Runs once, not measured)
fn setup_data() -> (Vec<u8>, [u8; 32], Vec<u8>) {
    let key = [1u8; 32];
    let mut data = vec![0u8; 1024]; // 1 KB
    let cipher_data = encrypt_data(&mut data, &key).unwrap();
    let (iv, ciphertext) = cipher_data.split_at(16);
    (iv.to_vec(), key, ciphertext.to_vec())
}

#[library_benchmark]
fn bench_encrypt_1kb() {
    let key = black_box([1u8; 32]);
    let mut data = black_box(vec![0u8; 1024]);
    let _ = encrypt_data(&mut data, &key).unwrap();
}

// Use the bench attribute to pass setup data into the function
#[library_benchmark]
#[bench::pure_decryption(setup_data())]
fn bench_decrypt_1kb(args: (Vec<u8>, [u8; 32], Vec<u8>)) {
    let (iv, key, mut ciphertext) = args;
    let _ = decrypt_data(black_box(&mut ciphertext), black_box(&key), black_box(&iv)).unwrap();
}

library_benchmark_group!(
    name = cipher_group;
    benchmarks = bench_encrypt_1kb, bench_decrypt_1kb
);

main!(library_benchmark_groups = cipher_group);
