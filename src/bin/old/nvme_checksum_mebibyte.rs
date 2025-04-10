use std::env;
#[cfg(target_arch = "aarch64")]
use crc_fast::crc64::reverse::aarch64::{update};

#[cfg(target_arch = "x86_64")]
use crc_fast::crc64::reverse::x86::{update};

use std::hint::black_box;
use rand::{rng, Rng};
use crc_fast::crc64::definitions::CRC64_NVME;

fn main() {
    let mut rng = rng();
    let data_len = 1024 * 1024;
    let iterations = 200000;

    println!("Hello, UPDATE SIMD world!");

    // if I comment out this next line, the performance goes from 50GiB/s to 63 GiB/s
    println!("iterations = {}", iterations);

    // Generate random data for this length
    let mut data = vec![0u8; data_len];
    rng.fill(&mut data[..]);

    println!("data_len = {}", data_len);

    let start = std::time::Instant::now();

    for _i in 0..iterations {
        unsafe {
            black_box(update(&data, CRC64_NVME.init, CRC64_NVME) ^ CRC64_NVME.xorout);
        }
    }

    let duration = start.elapsed();
    let elapsed_nanos = duration.as_nanos();
    let processed_data = data.len() as i64 * iterations;

    println!("{} iterations", iterations);
    //println!("data_len = {}", data_len);
    println!("{:.4} ns/iter", elapsed_nanos as f64 / iterations as f64);
    println!("{:.4} seconds", elapsed_nanos as f64 / 1_000_000_000.0);
    println!("{} bytes", processed_data);
    println!(
        "{:.4} MiB/s",
        processed_data as f64 / elapsed_nanos as f64 * 1_000_000_000.0 / 1024.0 / 1024.0
    );
    println!(
        "{:.4} GiB/s",
        processed_data as f64 / elapsed_nanos as f64 * 1_000_000_000.0 / 1024.0 / 1024.0 / 1024.0
    );
    println!();
    println!();
}