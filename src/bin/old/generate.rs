use crc::{CRC_32_AIXM, CRC_32_AUTOSAR, CRC_32_BASE91_D, CRC_32_CD_ROM_EDC, CRC_32_MEF, CRC_32_XFER, CRC_64_MS, CRC_64_REDIS};
use crc_fast::generate::keys;

fn main() {
    let config = CRC_64_REDIS;

    let keys = keys(config.width, config.poly as u64, config.refin);

    println!("const KEYS: [u64; 21] = [");

    for (i, key) in keys.iter().enumerate() {
        println!("0x{:016x},", key);
    }

    println!("];");
}