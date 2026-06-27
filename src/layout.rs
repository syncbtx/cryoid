pub trait Layout: Sized {
    const TIMESTAMP_BITS: u8;
    const TAG_BITS:       u8;
    const MACHINE_BITS:   u8;
    const SEQUENCE_BITS:  u8;
    const EPOCH:          u64;

    const MACHINE_SHIFT:u8 =    Self::SEQUENCE_BITS;
    const TAG_SHIFT: u8 =       Self::MACHINE_BITS + Self::SEQUENCE_BITS;
    const TIMESTAMP_SHIFT: u8 = Self::TAG_BITS + Self::MACHINE_BITS + Self::SEQUENCE_BITS;

    const MACHINE_MASK:   u64 = (1 << Self::MACHINE_BITS) - 1;
    const SEQUENCE_MASK:  u64 = (1 << Self::SEQUENCE_BITS) - 1;
    const TAG_MASK:       u64 = (1 << Self::TAG_BITS) - 1;
    const TIMESTAMP_MASK: u64 = (1 << Self::TIMESTAMP_BITS) - 1;
}