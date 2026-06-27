use crate::Layout;

// ── Decoded ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Decoded<T = u64> {
    pub timestamp: u64,
    pub tag:       T,
    pub machine:   u64,
    pub sequence:  u64,
}

impl<T: core::fmt::Debug> core::fmt::Debug for Decoded<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Decoded")
            .field("timestamp", &self.timestamp)
            .field("tag",       &self.tag)
            .field("machine",   &self.machine)
            .field("sequence",  &self.sequence)
            .finish()
    }
}

impl<T: core::fmt::Display> core::fmt::Display for Decoded<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ts={} tag={} machine={} seq={}",
            self.timestamp, self.tag, self.machine, self.sequence
        )
    }
}

// ── decode ────────────────────────────────────────────────────────────────────

/// Decompose a raw CryoID into its constituent fields.
///
/// Returns a bare `u64` tag — use this when decoding an ID that arrived
/// from outside the process (database, wire format) and there's no live
/// [`crate::Gen`] instance to call [`crate::Gen::decode`] on.
///
/// For in-process decoding where the generator that produced the ID is
/// still in scope, prefer `gen.decode(id)` — it resolves the tag straight
/// into its enum type with no extra type parameter, because the generator
/// already knows it.
#[inline(always)]
pub fn decode<L: Layout>(id: u64) -> Decoded<u64> {
    Decoded {
        timestamp: ((id >> L::TIMESTAMP_SHIFT) & L::TIMESTAMP_MASK) + L::EPOCH,
        tag:           (id >> L::TAG_SHIFT)     & L::TAG_MASK,
        machine:       (id >> L::MACHINE_SHIFT) & L::MACHINE_MASK,
        sequence:       id                       & L::SEQUENCE_MASK,
    }
}