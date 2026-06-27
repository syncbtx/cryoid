use crate::Layout;

#[derive(Debug, Clone,Copy, Eq, PartialEq)]
pub struct Tagged63;

impl Layout for Tagged63 {
    const TIMESTAMP_BITS: u8 = 41;
    const TAG_BITS:        u8 = 8;
    const MACHINE_BITS:    u8 = 5;
    const SEQUENCE_BITS:   u8 = 9;
    const EPOCH:       u64 = 1_704_067_200_000;
}