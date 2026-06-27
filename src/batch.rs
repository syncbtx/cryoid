/// An iterator over a contiguous block of CryoIDs sharing the same timestamp.
///
/// Obtained from [`Gen::batch`]. All boundary checks were performed at
/// construction — iteration is a single OR and increment per ID.
///
/// IDs are guaranteed to be monotonically increasing within the batch.
/// The iterator implements [`ExactSizeIterator`] and [`DoubleEndedIterator`].
#[derive(Debug, Clone,Copy, Eq, PartialEq)]
pub struct BatchIterator {
    pub(crate) base_id:          u64,
    pub(crate) current_sequence: u64,
    pub(crate) end_sequence:     u64, // exclusive
}

impl Iterator for BatchIterator {
    type Item = u64;

    #[inline(always)]
    fn next(&mut self) -> Option<u64> {
        if self.current_sequence < self.end_sequence {
            let id = self.base_id | self.current_sequence;
            self.current_sequence += 1;
            Some(id)
        } else {
            None
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = (self.end_sequence - self.current_sequence) as usize;
        (rem, Some(rem))
    }
}

impl ExactSizeIterator for BatchIterator {}

impl DoubleEndedIterator for BatchIterator {
    #[inline(always)]
    fn next_back(&mut self) -> Option<u64> {
        if self.current_sequence < self.end_sequence {
            self.end_sequence -= 1;
            Some(self.base_id | self.end_sequence)
        } else {
            None
        }
    }
}