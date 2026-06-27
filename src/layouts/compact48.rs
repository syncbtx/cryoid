use crate::Layout;

#[derive(Debug, Clone,Copy, Eq, PartialEq)]
pub struct Compact48;

impl Layout for Compact48 {
    const TIMESTAMP_BITS: u8 = 32;
    const TAG_BITS:        u8 = 0;
    const MACHINE_BITS:    u8 = 8;
    const SEQUENCE_BITS:   u8 = 8;
    const EPOCH:       u64 = 1_777_680_000_000;
}