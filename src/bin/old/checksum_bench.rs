#[cfg(target_arch = "aarch64")]
use crc_fast::crc64::arch::update;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use crc_fast::crc64::arch::update;

use crc_fast::crc64::definitions::CRC64_NVME;
use crc_fast::structs::CrcParams;
use rand::{rng, Rng};

type CalculatorFn = unsafe fn(
    &[u8],     // data
    u64,       // state
    CrcParams, // CRC implementation parameters
) -> u64;

fn main() {
    calculate("UPDATE SIMD", update, CRC64_NVME);
}

#[inline(always)]
fn calculate(name: &str, calculator: CalculatorFn, params: CrcParams) {
    let mut rng = rng();
    let iterations = 200000;
    let data_len = 1024 * 1024;

    println!("Hello, {} world!", name);

    // Generate random data for this length
    let mut data = vec![0u8; data_len as usize];
    rng.fill(&mut data[..]);

    let start = std::time::Instant::now();

    for _i in 0..iterations {
        unsafe {
            let checksum = calculator(&data, params.init, params) ^ params.xorout;

            if checksum == 0x0 {
                panic!("Checksum mismatch: {checksum:#x}");
            }
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
}
