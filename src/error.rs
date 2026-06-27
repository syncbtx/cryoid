#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CryoidError {
    SequenceOverflow {
        timestamp:    u64,
        max_sequence: u64,
    },
    ClockRegression {
        supplied:  u64,
        last_seen: u64,
    },
    PreEpochTimestamp {
        supplied: u64,
        epoch:    u64,
    },
    MachineIdOutOfRange {
        supplied: u64,
        max:      u64,
    },
    TagOutOfRange {
        supplied: u64,
        max:      u64,
    },
}

impl core::fmt::Debug for CryoidError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryoidError::SequenceOverflow { timestamp, max_sequence } => f
                .debug_struct("SequenceOverflow")
                .field("timestamp",    timestamp)
                .field("max_sequence", max_sequence)
                .finish(),

            CryoidError::ClockRegression { supplied, last_seen } => f
                .debug_struct("ClockRegression")
                .field("supplied",  supplied)
                .field("last_seen", last_seen)
                .finish(),

            CryoidError::PreEpochTimestamp { supplied, epoch } => f
                .debug_struct("PreEpochTimestamp")
                .field("supplied", supplied)
                .field("epoch",    epoch)
                .finish(),

            CryoidError::MachineIdOutOfRange { supplied, max } => f
                .debug_struct("MachineIdOutOfRange")
                .field("supplied", supplied)
                .field("max",      max)
                .finish(),

            CryoidError::TagOutOfRange { supplied, max } => f
                .debug_struct("TagOutOfRange")
                .field("supplied", supplied)
                .field("max",      max)
                .finish(),
        }
    }
}

impl core::fmt::Display for CryoidError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryoidError::SequenceOverflow { timestamp, max_sequence } => write!(
                f,
                "sequence overflow at timestamp {timestamp}: \
                 exhausted {max_sequence} slots in this millisecond"
            ),
            CryoidError::ClockRegression { supplied, last_seen } => write!(
                f,
                "clock regression: supplied timestamp {supplied} \
                 is behind last seen {last_seen}"
            ),
            CryoidError::PreEpochTimestamp { supplied, epoch } => write!(
                f,
                "pre-epoch timestamp: supplied {supplied} \
                 precedes layout epoch {epoch}"
            ),
            CryoidError::MachineIdOutOfRange { supplied, max } => write!(
                f,
                "machine ID out of range: supplied {supplied}, layout maximum {max}"
            ),
            CryoidError::TagOutOfRange { supplied, max } => write!(
                f,
                "tag out of range: supplied {supplied}, layout maximum {max}"
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CryoidError {}