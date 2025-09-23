// Copyright 2025 Don MacAskill. Licensed under MIT or Apache-2.0.

//! This module provides AArch64-specific implementations of the ArchOps trait.

#![cfg(target_arch = "aarch64")]

use crate::traits::ArchOps;
use aarch64::AArch64Ops;
use std::arch::aarch64::*;

use crate::arch::aarch64;

#[derive(Debug, Copy, Clone)]
pub struct AArch64Sha3Ops(AArch64Ops);

impl AArch64Sha3Ops {
    pub fn new() -> Self {
        AArch64Sha3Ops(AArch64Ops)
    }
}

impl ArchOps for AArch64Sha3Ops {
    type Vector = uint8x16_t;

    // Delegate methods to the base AES implementation
    #[inline(always)]
    unsafe fn create_vector_from_u64_pair(
        &self,
        high: u64,
        low: u64,
        reflected: bool,
    ) -> Self::Vector {
        self.0.create_vector_from_u64_pair(high, low, reflected)
    }

    #[inline(always)]
    unsafe fn create_vector_from_u64_pair_non_reflected(
        &self,
        high: u64,
        low: u64,
    ) -> Self::Vector {
        self.0.create_vector_from_u64_pair_non_reflected(high, low)
    }

    #[inline(always)]
    unsafe fn create_vector_from_u64(&self, value: u64, high: bool) -> Self::Vector {
        self.0.create_vector_from_u64(value, high)
    }

    #[inline(always)]
    unsafe fn extract_u64s(&self, vector: Self::Vector) -> [u64; 2] {
        self.0.extract_u64s(vector)
    }

    #[inline(always)]
    unsafe fn extract_poly64s(&self, vector: Self::Vector) -> [u64; 2] {
        self.0.extract_poly64s(vector)
    }

    #[inline(always)]
    unsafe fn xor_vectors(&self, a: Self::Vector, b: Self::Vector) -> Self::Vector {
        self.0.xor_vectors(a, b)
    }

    #[inline(always)]
    unsafe fn load_bytes(&self, ptr: *const u8) -> Self::Vector {
        self.0.load_bytes(ptr)
    }

    #[inline(always)]
    unsafe fn load_aligned(&self, ptr: *const [u64; 2]) -> Self::Vector {
        self.0.load_aligned(ptr)
    }

    #[inline(always)]
    unsafe fn shuffle_bytes(&self, data: Self::Vector, mask: Self::Vector) -> Self::Vector {
        self.0.shuffle_bytes(data, mask)
    }

    #[inline(always)]
    unsafe fn blend_vectors(
        &self,
        a: Self::Vector,
        b: Self::Vector,
        mask: Self::Vector,
    ) -> Self::Vector {
        self.0.blend_vectors(a, b, mask)
    }

    #[inline(always)]
    unsafe fn shift_left_8(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_left_8(vector)
    }

    #[inline(always)]
    unsafe fn set_all_bytes(&self, value: u8) -> Self::Vector {
        self.0.set_all_bytes(value)
    }

    #[inline(always)]
    unsafe fn create_compare_mask(&self, vector: Self::Vector) -> Self::Vector {
        self.0.create_compare_mask(vector)
    }

    #[inline(always)]
    unsafe fn and_vectors(&self, a: Self::Vector, b: Self::Vector) -> Self::Vector {
        self.0.and_vectors(a, b)
    }

    #[inline(always)]
    unsafe fn shift_right_32(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_right_32(vector)
    }

    #[inline(always)]
    unsafe fn shift_left_32(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_left_32(vector)
    }

    #[inline(always)]
    unsafe fn create_vector_from_u32(&self, value: u32, high: bool) -> Self::Vector {
        self.0.create_vector_from_u32(value, high)
    }

    #[inline(always)]
    unsafe fn shift_left_4(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_left_4(vector)
    }

    #[inline(always)]
    unsafe fn shift_right_4(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_right_4(vector)
    }

    #[inline(always)]
    unsafe fn shift_right_8(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_right_8(vector)
    }

    #[inline(always)]
    unsafe fn shift_right_5(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_right_5(vector)
    }

    #[inline(always)]
    unsafe fn shift_right_6(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_right_6(vector)
    }

    #[inline(always)]
    unsafe fn shift_right_7(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_right_7(vector)
    }

    #[inline(always)]
    unsafe fn shift_right_12(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_right_12(vector)
    }

    #[inline(always)]
    unsafe fn shift_left_12(&self, vector: Self::Vector) -> Self::Vector {
        self.0.shift_left_12(vector)
    }

    #[inline(always)]
    unsafe fn carryless_mul_00(&self, a: Self::Vector, b: Self::Vector) -> Self::Vector {
        self.0.carryless_mul_00(a, b)
    }

    #[inline(always)]
    unsafe fn carryless_mul_01(&self, a: Self::Vector, b: Self::Vector) -> Self::Vector {
        self.0.carryless_mul_01(a, b)
    }

    #[inline(always)]
    unsafe fn carryless_mul_10(&self, a: Self::Vector, b: Self::Vector) -> Self::Vector {
        self.0.carryless_mul_10(a, b)
    }

    #[inline(always)]
    unsafe fn carryless_mul_11(&self, a: Self::Vector, b: Self::Vector) -> Self::Vector {
        self.0.carryless_mul_11(a, b)
    }

    #[inline]
    #[cfg(target_feature = "sha3")]
    #[target_feature(enable = "sha3")]
    unsafe fn xor3_vectors(
        &self,
        a: Self::Vector,
        b: Self::Vector,
        c: Self::Vector,
    ) -> Self::Vector {
        veor3q_u8(a, b, c)
    }

    #[inline]
    #[cfg(not(target_feature = "sha3"))]
    #[target_feature(enable = "neon")]
    unsafe fn xor3_vectors(
        &self,
        a: Self::Vector,
        b: Self::Vector,
        c: Self::Vector,
    ) -> Self::Vector {
        // Fallback for when SHA3 is not available
        veorq_u8(veorq_u8(a, b), c)
    }
}
