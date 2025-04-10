#[cfg(target_arch = "aarch64")]
use crc_fast::crc64::reverse::aarch64::{update};

#[cfg(target_arch = "x86_64")]
use crc_fast::crc64::reverse::x86::{update};

use std::hint::black_box;
use std::env;
use std::process::ExitCode;
use rand::{rng, Rng};
use crc_fast::crc64::definitions::CRC64_NVME;
use crc_fast::structs::CrcParams;

type CalculatorFn = unsafe fn(
    &[u8],     // data
    u64,       // state
    CrcParams, // CRC implementation parameters
) -> u64;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let (iterations, data_len) = if args.len() == 1 {
        println!("Usage: checksum_bench [iterations] [data_len]");
        println!("Example for a 1MiB buffer and 200000 iterations: checksum_bench 200000 1048576");
        println!("Using default values of 200000 iterations and 1MiB buffer.");
        println!("");

        (200000, 1048576)
    } else {
        println!("");
        (args[2].parse().unwrap(), args[3].parse().unwrap())
    };

    let mut rng = rng();

    println!("Hello, WTF world!");

    // Generate random data for this length
    let mut data = vec![0u8; data_len as usize];
    rng.fill(&mut data[..]);

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

    ExitCode::from(0)
}