use crate::Layout;

#[derive(Debug, Clone,Copy, Eq, PartialEq)]
pub struct Snowflake63;

impl Layout for Snowflake63 {
    const TIMESTAMP_BITS: u8 = 41;
    const TAG_BITS:        u8 = 0;
    const MACHINE_BITS:    u8 = 10;
    const SEQUENCE_BITS:   u8 = 12;
    const EPOCH:       u64 = 1_704_067_200_000;
}