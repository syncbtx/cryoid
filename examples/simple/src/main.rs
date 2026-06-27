use cryoid::{decode, Gen, Layout, layouts::{Compact48, Snowflake63, Tagged63}};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct LedgerLayout;

impl Layout for LedgerLayout {
    const TIMESTAMP_BITS: u8 = 40;
    const TAG_BITS:        u8 = 8;
    const MACHINE_BITS:    u8 = 6;
    const SEQUENCE_BITS:   u8 = 9;
    const EPOCH:          u64 = 1_704_067_200_000;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Entity {
    Order       = 0,
    Transaction = 1,
    Audit       = 2,
}

impl From<Entity> for u64 {
    fn from(v: Entity) -> u64 {
        v as u64
    }
}

impl TryFrom<u64> for Entity {
    type Error = ();
    fn try_from(v: u64) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Entity::Order),
            1 => Ok(Entity::Transaction),
            2 => Ok(Entity::Audit),
            _ => Err(()),
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn section(title: &str) {
    println!("\n── {title} {}", "─".repeat(60usize.saturating_sub(title.len())));
}

fn main() {
    println!("=== CryoID Showcase ===");


    section("Typed tags across multiple generators");

    let mut order_gen = Gen::<LedgerLayout>::builder()
        .machine(0)
        .with_tag(Entity::Order)
        .build()
        .unwrap();
    let mut tx_gen = Gen::<LedgerLayout>::builder()
        .machine(1)
        .with_tag(Entity::Transaction)
        .build()
        .unwrap();
    let mut audit_gen = Gen::<LedgerLayout>::builder()
        .machine(2)
        .with_tag(Entity::Audit)
        .build()
        .unwrap();

    let ts = now_ms();
    let order_id = order_gen.next_id(ts).unwrap();
    let tx_id    = tx_gen.next_id(ts).unwrap();
    let audit_id = audit_gen.next_id(ts).unwrap();

    for (label, gen, id) in [
        ("order", &order_gen, order_id),
        ("transaction", &tx_gen, tx_id),
        ("audit", &audit_gen, audit_id),
    ] {
        let d = gen.decode(id).unwrap();
        println!("  {label:<12} id={id:<22} kind={:?} machine={} seq={}", d.tag, d.machine, d.sequence);
    }

    section("Causal ordering: events sort by id, not by stream");

    let mut events: Vec<(&str, u64)> = vec![
        ("order",       order_id),
        ("transaction", tx_id),
        ("audit",       audit_id),
    ];
    events.sort_by_key(|(_, id)| *id);
    for (label, id) in &events {
        println!("  {id:<22} <- {label}");
    }

    section("Sequential generation within one millisecond");

    let mut gen = Gen::<LedgerLayout>::builder().machine(3).build().unwrap();
    let ts = now_ms();
    let ids: Vec<u64> = (0..5).map(|_| gen.next_id(ts).unwrap()).collect();
    for id in &ids {
        println!("  id={id} seq={}", decode::<LedgerLayout>(*id).sequence);
    }
    assert!(ids.windows(2).all(|w| w[0] < w[1]));

    section("Tick rollover resets sequence");

    let next_id = gen.next_id(ts + 1).unwrap();
    let d = decode::<LedgerLayout>(next_id);
    println!("  id={next_id} ts={} seq={}", d.timestamp, d.sequence);
    assert_eq!(d.sequence, 0);

    section("Batch reservation, forward and reverse");

    let batch_ts = ts + 2;
    let mut batch = gen.batch(batch_ts, 16).unwrap();

    print!("  forward: ");
    for id in batch.by_ref().take(4) {
        print!("{} ", decode::<LedgerLayout>(id).sequence);
    }
    println!();

    print!("  reverse: ");
    for id in batch.rev().take(4) {
        print!("{} ", decode::<LedgerLayout>(id).sequence);
    }
    println!();

    section("Clock regression is rejected");

    match gen.next_id(batch_ts - 1) {
        Ok(_)  => println!("  unexpected success"),
        Err(e) => println!("  {e}"),
    }

    section("Pre-epoch timestamps are rejected");

    match gen.next_id(LedgerLayout::EPOCH - 1) {
        Ok(_)  => println!("  unexpected success"),
        Err(e) => println!("  {e}"),
    }

    section("Sequence exhaustion at the exact boundary");

    let exhaust_ts    = batch_ts + 1;
    let full_capacity = LedgerLayout::SEQUENCE_MASK + 1;

    let exact = gen.batch(exhaust_ts, full_capacity);
    println!(
        "  request exactly {full_capacity} (fresh tick capacity) -> {}",
        if exact.is_ok() { "ok" } else { "rejected" }
    );

    let mut gen2 = Gen::<LedgerLayout>::builder().machine(3).build().unwrap();
    let over = gen2.batch(exhaust_ts, full_capacity + 1);
    match over {
        Ok(_)  => println!("  unexpected success"),
        Err(e) => println!("  request one past capacity -> {e}"),
    }

    let recovery_id = gen2.next_id(exhaust_ts).unwrap();
    println!("  state survives: {}", decode::<LedgerLayout>(recovery_id));

    section("Machine and tag boundary rejection");

    let bad_machine = Gen::<LedgerLayout>::builder()
        .machine(LedgerLayout::MACHINE_MASK + 1)
        .build();
    let bad_tag = Gen::<LedgerLayout>::builder()
        .with_tag(LedgerLayout::TAG_MASK + 1)
        .build();
    println!("  machine over limit -> {:?}", bad_machine.err().unwrap());
    println!("  tag over limit     -> {:?}", bad_tag.err().unwrap());

    section("Decoding rejects unrecognized tag bits");

    let corrupted = order_id | (200u64 << LedgerLayout::TAG_SHIFT);
    match order_gen.decode(corrupted) {
        Some(_) => println!("  unexpected success"),
        None    => println!("  rejected: tag value 200 has no Entity variant"),
    }

    section("Different machines never collide");

    let mut a = Gen::<Snowflake63>::builder().machine(0).build().unwrap();
    let mut b = Gen::<Snowflake63>::builder().machine(1).build().unwrap();
    let collide_ts = now_ms();
    let id_a = a.next_id(collide_ts).unwrap();
    let id_b = b.next_id(collide_ts).unwrap();
    println!("  a={id_a} b={id_b} equal={}", id_a == id_b);

    section("Compact48 for embedded / UDP packet ids");

    let mut packet_gen = Gen::<Compact48>::builder().machine(0).build().unwrap();
    let packets: Vec<u64> = packet_gen.batch(now_ms(), 6).unwrap().collect();
    for id in &packets {
        println!("  {id}");
    }
    println!("  fits in 48 bits: {}", packets.iter().all(|&id| id < (1u64 << 48)));

    section("Tagged63 preset reused with a different enum");

    #[derive(Debug, Clone, Copy)]
    #[repr(u64)]
    enum OrderKind {
        Market = 0,
        Limit  = 1,
    }
    impl From<OrderKind> for u64 {
        fn from(v: OrderKind) -> u64 { v as u64 }
    }
    impl TryFrom<u64> for OrderKind {
        type Error = ();
        fn try_from(v: u64) -> Result<Self, Self::Error> {
            match v {
                0 => Ok(OrderKind::Market),
                1 => Ok(OrderKind::Limit),
                _ => Err(()),
            }
        }
    }

    let mut market_gen = Gen::<Tagged63>::builder()
        .machine(0)
        .with_tag(OrderKind::Market)
        .build()
        .unwrap();
    let market_id = market_gen.next_id(now_ms()).unwrap();
    let market_d  = market_gen.decode(market_id).unwrap();
    println!("  same Tagged63 layout, different enum -> kind={:?}", market_d.tag);
}