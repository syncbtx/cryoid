use cryoid::{Gen, layouts::Snowflake63, decode};
fn current_timestamp() -> u64 {
    1_710_000_000_000 // A mocked millisecond timestamp
}

fn main() {
    // Create a generator for machine #42 using the standard Snowflake63 layout
    let mut gen = Gen::<Snowflake63>::builder()
        .machine(42)
        .build()
        .expect("Failed to build generator");

    let timestamp = current_timestamp();

    // Generate a new ID
    let id = gen.next_id(timestamp).expect("Failed to generate ID");
    println!("Generated ID: {}", id);

    // Decode the ID back into its components
    let decoded = decode::<Snowflake63>(id);
    println!("Decoded: machine={}, sequence={}, timestamp={}",
        decoded.machine,
        decoded.sequence,
        decoded.timestamp
    );

    let batch = gen.batch(timestamp, 16).expect("Failed to generate batch");
    println!("Batch: {:?}", batch);
    for id in batch {
        println!("ID: {:?}", id);
    }
}
