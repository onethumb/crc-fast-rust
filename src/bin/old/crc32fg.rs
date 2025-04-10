// crc32fg.rs = generate constant table

// CRC-32/ISO-HDLC
const RK08: u64 = 0x0000000104c11db7;

fn main() {
    let prk: [u64; 21] = [
        0,
        32 * 3,
        32 * 5,
        32 * 31,
        32 * 33,
        32 * 3,
        32 * 2,
        0,
        0,
        32 * 27,
        32 * 29,
        32 * 23,
        32 * 25,
        32 * 19,
        32 * 21,
        32 * 15,
        32 * 17,
        32 * 11,
        32 * 13,
        32 * 7,
        32 * 9,
    ];

    let mut crk: [u64; 21] = [0; 21];
    crk[0] = 0; // crk[0] not used

    for i in 1..21 {
        crk[i] = grk(prk[i]);
    }

    crk[7] = grk07(); // rk07 = 2^64 / rk08 (using xor divide)
    crk[8] = RK08; // rk08 = polynomial

    for i in 1..21 {
        println!("rk{:02}    dq      0{:016x}h", i, crk[i]);
    }
}

fn grk07() -> u64 {
    let mut n: u64 = 0x100000000;
    let mut q: u64 = 0;

    for _ in 0..33 {
        q <<= 1;
        if n & 0x100000000 != 0 {
            q |= 1;
            n ^= RK08;
        }
        n <<= 1;
    }

    q
}

fn grk(e: u64) -> u64 {
    if e < 32 {
        return 0;
    }

    let mut n: u64 = 0x080000000;
    let e = e - 31;

    for _ in 0..e {
        n <<= 1;
        if n & 0x100000000 != 0 {
            n ^= RK08;
        }
    }

    n << 32
}
