use crc_fast::enums::CrcAlgorithm;
use crc_fast::{checksum, get_calculator_target};

fn main() {
    println!(
        "{} = {:x}",
        get_calculator_target(CrcAlgorithm::Crc32IsoHdlc),
        checksum(CrcAlgorithm::Crc32IsoHdlc, b"123456789")
    );
    println!(
        "{} = {:x}",
        get_calculator_target(CrcAlgorithm::Crc32Iscsi),
        checksum(CrcAlgorithm::Crc32Iscsi, b"123456789")
    );
}
