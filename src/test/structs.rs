// Copyright 2025 Don MacAskill. Licensed under MIT or Apache-2.0.

#![cfg(test)]
#![allow(dead_code)]

use crate::structs::CrcParams;
use crc::Crc;

pub struct CrcTestConfig<T: crc::Width> {
    pub params: CrcParams,
    pub reference_impl: &'static Crc<T>,
}

pub type Crc32TestConfig = CrcTestConfig<u32>;

pub type Crc64TestConfig = CrcTestConfig<u64>;
