#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use crate::{decode, Gen, CryoidError, layouts::{Compact48, Snowflake63, Tagged63}, Layout};

    const TS: u64 = 1_704_067_200_000;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u64)]
    enum Entity {
        Order       = 0,
        Transaction = 1,
    }

    impl From<Entity> for u64 {
        fn from(v: Entity) -> u64 { v as u64 }
    }

    impl TryFrom<u64> for Entity {
        type Error = ();
        fn try_from(v: u64) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(Entity::Order),
                1 => Ok(Entity::Transaction),
                _ => Err(()),
            }
        }
    }

    // ── Construction ──────────────────────────────────────────────────────

    #[test]
    fn builder_defaults_to_machine_zero() {
        let e = Gen::<Snowflake63>::builder().build().unwrap();
        assert_eq!(e.sequence(), 0);
        assert!(e.last_timestamp().is_none());
    }

    #[test]
    fn builder_rejects_machine_overflow() {
        let err = Gen::<Snowflake63>::builder()
            .machine(1024)
            .build()
            .unwrap_err();
        assert!(matches!(err, CryoidError::MachineIdOutOfRange { .. }));
    }

    #[test]
    fn builder_rejects_tag_overflow() {
        let err = Gen::<Tagged63>::builder()
            .with_tag(256u64)
            .build()
            .unwrap_err();
        assert!(matches!(err, CryoidError::TagOutOfRange { .. }));
    }

    // ── next_id ───────────────────────────────────────────────────────────

    #[test]
    fn first_id_has_sequence_zero() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        let id = e.next_id(TS).unwrap();
        let d  = decode::<Snowflake63>(id);
        assert_eq!(d.sequence,  0);
        assert_eq!(d.timestamp, TS);
        assert_eq!(d.machine,   0);
    }

    #[test]
    fn same_tick_increments_sequence() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        let id0 = e.next_id(TS).unwrap();
        let id1 = e.next_id(TS).unwrap();
        let id2 = e.next_id(TS).unwrap();
        assert_eq!(decode::<Snowflake63>(id0).sequence, 0);
        assert_eq!(decode::<Snowflake63>(id1).sequence, 1);
        assert_eq!(decode::<Snowflake63>(id2).sequence, 2);
        assert!(id0 < id1 && id1 < id2);
    }

    #[test]
    fn new_tick_resets_sequence() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        e.next_id(TS).unwrap();
        e.next_id(TS).unwrap();
        let id = e.next_id(TS + 1).unwrap();
        assert_eq!(decode::<Snowflake63>(id).sequence,  0);
        assert_eq!(decode::<Snowflake63>(id).timestamp, TS + 1);
    }

    #[test]
    fn ids_are_monotonic_across_ticks() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        let mut prev = 0u64;
        for delta in 0..10u64 {
            for _ in 0..4 {
                let id = e.next_id(TS + delta).unwrap();
                assert!(id > prev, "id={id} not greater than prev={prev}");
                prev = id;
            }
        }
    }

    #[test]
    fn clock_regression_returns_error() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        e.next_id(TS + 5).unwrap();
        let err = e.next_id(TS + 4).unwrap_err();
        assert!(matches!(err, CryoidError::ClockRegression { .. }));
    }

    #[test]
    fn sequence_overflow_returns_error() {
        let mut e = Gen::<Compact48>::builder().build().unwrap();
        for i in 0..=255u64 {
            e.next_id(TS).unwrap_or_else(|_| panic!("failed at seq {i}"));
        }
        let err = e.next_id(TS).unwrap_err();
        assert!(matches!(err, CryoidError::SequenceOverflow { .. }));
    }

    #[test]
    fn pre_epoch_timestamp_returns_error() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        let err = e.next_id(Snowflake63::EPOCH - 1).unwrap_err();
        assert!(matches!(err, CryoidError::PreEpochTimestamp { .. }));
    }

    #[test]
    fn pre_epoch_does_not_mutate_state() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        let _ = e.next_id(Snowflake63::EPOCH - 1);
        assert!(e.last_timestamp().is_none());
        assert_eq!(e.sequence(), 0);

        let id = e.next_id(Snowflake63::EPOCH).unwrap();
        assert_eq!(decode::<Snowflake63>(id).sequence,  0);
        assert_eq!(decode::<Snowflake63>(id).timestamp, Snowflake63::EPOCH);
    }

    // ── batch ─────────────────────────────────────────────────────────────

    #[test]
    fn batch_yields_correct_count() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        let batch = e.batch(TS, 64).unwrap();
        assert_eq!(batch.len(), 64);
    }

    #[test]
    fn batch_ids_are_sorted() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        let ids: Vec<u64> = e.batch(TS, 16).unwrap().collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn batch_sequence_continues_from_next_id() {
        let mut e = Gen::<Snowflake63>::builder().build().unwrap();
        e.next_id(TS).unwrap();
        e.next_id(TS).unwrap();
        let batch: Vec<u64> = e.batch(TS, 4).unwrap().collect();
        assert_eq!(decode::<Snowflake63>(batch[0]).sequence, 2);
        assert_eq!(decode::<Snowflake63>(batch[3]).sequence, 5);
    }

    #[test]
    fn batch_overflow_returns_error() {
        let mut e = Gen::<Compact48>::builder().build().unwrap();
        let err = e.batch(TS, 256).unwrap_err();
        assert!(matches!(err, CryoidError::SequenceOverflow { .. }));
    }

    #[test]
    fn batch_exact_fresh_tick_capacity_succeeds() {
        let mut e = Gen::<Compact48>::builder().build().unwrap();
        let full = Compact48::SEQUENCE_MASK + 1; // 256 — exactly fills a fresh tick
        assert!(e.batch(TS, full).is_ok());
    }

    #[test]
    fn batch_one_past_fresh_tick_capacity_fails() {
        let mut e = Gen::<Compact48>::builder().build().unwrap();
        let over = Compact48::SEQUENCE_MASK + 2; // 257 — one past capacity
        let err = e.batch(TS, over).unwrap_err();
        assert!(matches!(err, CryoidError::SequenceOverflow { .. }));
    }

    #[test]
    fn batch_double_ended() {
        let mut e  = Gen::<Snowflake63>::builder().build().unwrap();
        let mut it = e.batch(TS, 4).unwrap();
        assert_eq!(decode::<Snowflake63>(it.next().unwrap()).sequence,      0);
        assert_eq!(decode::<Snowflake63>(it.next_back().unwrap()).sequence, 3);
        assert_eq!(decode::<Snowflake63>(it.next().unwrap()).sequence,      1);
        assert_eq!(decode::<Snowflake63>(it.next_back().unwrap()).sequence, 2);
        assert!(it.next().is_none());
    }

    // ── decode (free function, untyped tag) ──────────────────────────────

    #[test]
    fn round_trip_snowflake63() {
        let mut e = Gen::<Snowflake63>::builder().machine(7).build().unwrap();
        let id    = e.next_id(TS + 999).unwrap();
        let d     = decode::<Snowflake63>(id);
        assert_eq!(d.timestamp, TS + 999);
        assert_eq!(d.machine,   7);
        assert_eq!(d.sequence,  0);
        assert_eq!(d.tag,       0);
    }

    // ── Gen::decode (typed tag) ───────────────────────────────────────────

    #[test]
    fn gen_decode_resolves_typed_tag() {
        let mut e = Gen::<Tagged63>::builder()
            .machine(1)
            .with_tag(Entity::Order)
            .build()
            .unwrap();

        let id = e.next_id(TS).unwrap();
        let d  = e.decode(id).unwrap();

        assert_eq!(d.tag,       Entity::Order);
        assert_eq!(d.machine,   1);
        assert_eq!(d.timestamp, TS);
        assert_eq!(d.sequence,  0);
    }

    #[test]
    fn gen_decode_rejects_foreign_tag_bits() {
        // Manually craft an id with a tag value the Entity enum can't represent.
        let mut e = Gen::<Tagged63>::builder()
            .machine(0)
            .with_tag(Entity::Order)
            .build()
            .unwrap();
        let id = e.next_id(TS).unwrap();

        // Corrupt the tag field bits to a value Entity has no variant for.
        let corrupted = id | (1u64 << Tagged63::TAG_SHIFT);
        assert!(e.decode(corrupted).is_none());
    }

    #[test]
    fn different_machine_ids_never_collide() {
        let mut e0 = Gen::<Snowflake63>::builder().machine(0).build().unwrap();
        let mut e1 = Gen::<Snowflake63>::builder().machine(1).build().unwrap();
        for _ in 0..100 {
            assert_ne!(e0.next_id(TS).unwrap(), e1.next_id(TS).unwrap());
        }
    }
}