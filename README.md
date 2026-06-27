# <mark style="background-color:#1E6299">`cryoid`</mark>

<hr style="height:1px; border-width:0; background-color:grey; background-image:linear-gradient(to right, rgba(0,0,0,0), rgba(0,0,0,0.75), rgba(0,0,0,0));">

<small>

*If you need to ask a clock for the time, you've already lost control.*

`cryoid` is a product of stripping the standard [Snowflake ID](https://en.wikipedia.org/wiki/Snowflake_ID) algorithm down to its absolute bare minimum to fully harness its potential for modern, highly constrained applications.

Where most implementations rely on implicit system clocks and runtime allocations, `cryoid` demands that you provide the time — giving you back complete determinism, testability, and safety without ever leaving the stack.

Not all environments have the luxury of an operating system clock, so how do you generate unique, sequential identifiers in high-frequency trading, simulations, or embedded devices?  
That is where standard Snowflake generators stall and `cryoid` excels.  

`cryoid` bridges the gap, allowing you to design and implement your own compile-time bit layouts and epoch boundaries natively, keeping you in complete control of the layout and the clock.

`cryoid` can comfortably do everything the standard Snowflake algorithm is capable of and more — with compile-time safety, strict zero allocations, zero system calls, and an uncompromising developer experience.

### Architecture

* **`Zero-Allocation & no_std`:** Runs entirely on the stack, with no operating system dependencies.
* **`Caller-Injected Time`:**  Bring your own clock for full determinism and a rock-solid defense against clock regression.
* **`Bit Layouts at Compile Time`:** Native bit distribution and epochs through your own definition of the Layout trait.
* **`The Typestate Builder`:**  The runtime does not impose structural constraints or domain-specific tags . The compiler does .
* **`Batching Without Friction`:** Validation is performed exactly once, reducing iteration to a single atomic bitwise OR.
* **`Type-safe decode`:** Turn the generation inside out to safely resurrect timestamps and cast raw tags back into domain-specific types.


### Example
```rust
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

```
</small>
