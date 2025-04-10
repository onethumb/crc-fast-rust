// crc64fg.rs = generate constant table

// CRC-64/ECMA-182
const RK08: u64 = 0x42F0E1EBA9EA3693;

fn main() {
    let prk: [u64; 21] = [
        0,
        64 * 2,
        64 * 3,
        64 * 16,
        64 * 17,
        64 * 2,
        64,
        0,
        0,
        64 * 14,
        64 * 15,
        64 * 12,
        64 * 13,
        64 * 10,
        64 * 11,
        64 * 8,
        64 * 9,
        64 * 6,
        64 * 7,
        64 * 4,
        64 * 5,
    ];

    let mut crk: [u64; 21] = [0; 21];
    crk[0] = 0; // crk[0] not used

    for i in 1..21 {
        crk[i] = grk(prk[i]);
    }

    crk[7] = grk07(); // rk07 = 2^128 / poly
    crk[8] = RK08; // rk08 = poly-2^64

    for i in 1..21 {
        println!("rk{:02}    dq      0{:016x}h", i, crk[i]);
    }
}

fn grk07() -> u64 {
    let mut n_hi: u64 = 0x0000000000000001;
    let mut n_lo: u64 = 0x0000000000000000;
    let mut q: u64 = 0;

    for _ in 0..65 {
        q <<= 1;
        if n_hi != 0 {
            q |= 1;
            n_lo ^= RK08;
        }
        n_hi = n_lo >> 63;
        n_lo <<= 1;
    }

    q // 2^128/poly
}

fn grk(e: u64) -> u64 {
    if e <= 64 {
        return 0;
    }

    let mut n: u64 = 0x8000000000000000;
    let e = e - 63;

    for _ in 0..e {
        n = (n << 1) ^ ((0_u64.wrapping_sub(n >> 63)) & RK08);
    }

    n // 2^(E)%poly
}
