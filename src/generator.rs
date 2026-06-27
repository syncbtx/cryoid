use core::marker::PhantomData;
use crate::{BatchIterator, CryoidError, Decoded, Layout};

// ── Typestate markers ─────────────────────────────────────────────────────────

pub struct NoTag;
pub struct WithTag<T>(T);

// ── Builder ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone,Copy, Eq, PartialEq)]
pub struct GenBuilder<L: Layout, T = NoTag> {
    machine: u64,
    tag:     T,
    _layout: PhantomData<L>,
}

impl<L: Layout> GenBuilder<L, NoTag> {
    pub fn new() -> Self {
        Self {
            machine: 0,
            tag:     NoTag,
            _layout: PhantomData,
        }
    }

    pub fn machine(mut self, id: u64) -> Self {
        self.machine = id;
        self
    }

    pub fn with_tag<T>(self, value: T) -> GenBuilder<L, WithTag<T>>
    where
        T: Into<u64>,
    {
        GenBuilder {
            machine: self.machine,
            tag:     WithTag(value),
            _layout: PhantomData,
        }
    }

    pub fn build(self) -> Result<Gen<L, u64>, CryoidError> {
        Gen::from_parts(self.machine, 0)
    }
}

impl<L: Layout, T: Into<u64>> GenBuilder<L, WithTag<T>> {
    pub fn machine(mut self, id: u64) -> Self {
        self.machine = id;
        self
    }

    pub fn build(self) -> Result<Gen<L, T>, CryoidError> {
        let raw = self.tag.0.into();
        Gen::from_parts(self.machine, raw)
    }
}

// ── Gen ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone,Copy, Eq, PartialEq)]
pub struct Gen<L: Layout, T = u64> {
    static_prefix:  u64,
    last_timestamp: u64,
    sequence:       u64,
    _layout:        PhantomData<L>,
    _tag:           PhantomData<T>,
}

// ── Construction ──────────────────────────────────────────────────────────────

impl<L: Layout> Gen<L, u64> {
    pub fn builder() -> GenBuilder<L, NoTag> {
        GenBuilder::new()
    }
}

impl<L: Layout, T> Gen<L, T> {
    fn from_parts(machine: u64, tag: u64) -> Result<Self, CryoidError> {
        #[inline(always)]
        fn const_assert_layout<L: Layout>() {
            const { assert!(
                L::TIMESTAMP_BITS as u16
                    + L::TAG_BITS      as u16
                    + L::MACHINE_BITS  as u16
                    + L::SEQUENCE_BITS as u16
                    <= 63,
                "CryoID: bit layout must not exceed 63 bits to fit in a positive i64"
            ) };
        }
        const_assert_layout::<L>();

        if machine > L::MACHINE_MASK {
            return Err(CryoidError::MachineIdOutOfRange {
                supplied: machine,
                max:      L::MACHINE_MASK,
            });
        }

        if tag > L::TAG_MASK {
            return Err(CryoidError::TagOutOfRange {
                supplied: tag,
                max:      L::TAG_MASK,
            });
        }

        Ok(Self {
            static_prefix:  (tag << L::TAG_SHIFT) | (machine << L::MACHINE_SHIFT),
            last_timestamp: u64::MAX,
            sequence:       0,
            _layout:        PhantomData,
            _tag:           PhantomData,
        })
    }
}

// ── ID generation ─────────────────────────────────────────────────────────────

impl<L: Layout, T> Gen<L, T> {
    #[inline(always)]
    pub fn next_id(&mut self, timestamp: u64) -> Result<u64, CryoidError> {
        self.check_epoch(timestamp)?;
        self.check_regression(timestamp)?;

        if timestamp == self.last_timestamp {
            self.sequence += 1;
            if self.sequence > L::SEQUENCE_MASK {
                return Err(CryoidError::SequenceOverflow {
                    timestamp,
                    max_sequence: L::SEQUENCE_MASK,
                });
            }
        } else {
            self.sequence       = 0;
            self.last_timestamp = timestamp;
        }

        Ok(self.compose(timestamp, self.sequence))
    }

    pub fn batch(&mut self, timestamp: u64, count: u64) -> Result<BatchIterator, CryoidError> {
        self.check_epoch(timestamp)?;
        self.check_regression(timestamp)?;

        let start_sequence = if timestamp == self.last_timestamp {
            self.sequence + 1
        } else {
            0
        };

        let end_sequence = start_sequence + count;

        if end_sequence.saturating_sub(1) > L::SEQUENCE_MASK {
            return Err(CryoidError::SequenceOverflow {
                timestamp,
                max_sequence: L::SEQUENCE_MASK,
            });
        }

        self.last_timestamp = timestamp;
        self.sequence       = end_sequence - 1;

        Ok(BatchIterator {
            base_id:          self.compose(timestamp, 0),
            current_sequence: start_sequence,
            end_sequence,
        })
    }
}

// ── Decoding ──────────────────────────────────────────────────────────────────

impl<L: Layout, T: TryFrom<u64>> Gen<L, T> {
    /// Decode an ID this generator produced, resolving the tag directly
    /// into `T` — no extra type parameter needed, `Gen` already knows it.
    ///
    /// Returns `None` if the raw tag bits don't correspond to a valid `T`.
    pub fn decode(&self, id: u64) -> Option<Decoded<T>> {
        let raw_tag = (id >> L::TAG_SHIFT) & L::TAG_MASK;
        Some(Decoded {
            timestamp: ((id >> L::TIMESTAMP_SHIFT) & L::TIMESTAMP_MASK) + L::EPOCH,
            tag:       T::try_from(raw_tag).ok()?,
            machine:   (id >> L::MACHINE_SHIFT) & L::MACHINE_MASK,
            sequence:   id                       & L::SEQUENCE_MASK,
        })
    }
}

// ── Inspection ────────────────────────────────────────────────────────────────

impl<L: Layout, T> Gen<L, T> {
    #[inline(always)]
    pub fn last_timestamp(&self) -> Option<u64> {
        if self.last_timestamp == u64::MAX { None } else { Some(self.last_timestamp) }
    }

    #[inline(always)]
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

impl<L: Layout, T> Gen<L, T> {
    #[inline(always)]
    fn check_epoch(&self, timestamp: u64) -> Result<(), CryoidError> {
        if timestamp < L::EPOCH {
            return Err(CryoidError::PreEpochTimestamp {
                supplied: timestamp,
                epoch:    L::EPOCH,
            });
        }
        Ok(())
    }

    #[inline(always)]
    fn check_regression(&self, timestamp: u64) -> Result<(), CryoidError> {
        if self.last_timestamp != u64::MAX && timestamp < self.last_timestamp {
            return Err(CryoidError::ClockRegression {
                supplied:  timestamp,
                last_seen: self.last_timestamp,
            });
        }
        Ok(())
    }

    #[inline(always)]
    fn compose(&self, timestamp: u64, sequence: u64) -> u64 {
        let delta = timestamp - L::EPOCH;
        let ts = delta & L::TIMESTAMP_MASK;
        (ts << L::TIMESTAMP_SHIFT) | self.static_prefix | sequence
    }
}