# Revfl

[![Crates.io](https://img.shields.io/crates/v/revfl.svg)](https://crates.io/crates/revfl)
[![Documentation](https://docs.rs/revfl/badge.svg)](https://docs.rs/revfl)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[中文说明](README_zh.md)

A Rust library for parsing and writing Nintendo BFEVFL (Binary Format Event Flow) and BFEVTM (Binary Format Event Timeline) files, commonly used in Nintendo games (e.g., The Legend of Zelda: Breath of the Wild, Animal Crossing: New Horizons) for logic and cutscene scripting.

This library allows you to read, modify, and serialize these event files programmatically.

## Features

- **Full BFEVFL/BFEVTM support:** Read and write Event Flow and Event Timeline files.
- **Serialization:** Full `serde` support for easy conversion to/from JSON or other formats.
- **Preservation of structure:** Exact byte-for-byte repacking capability for unmodified files.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
revfl = "0.1.0"
```

## Usage

### Reading and Writing a BFEVFL File

```rust
use revfl::evfl::EventFlow;
use std::fs;
use std::io::Cursor;

fn main() {
    // 1. Read the BFEVFL file
    let data = fs::read("example.bfevfl").expect("Failed to read file");
    
    // 2. Parse the event flow
    let mut evfl = EventFlow::new();
    evfl.read(&data);
    
    // You can access flowcharts, timelines, actors, events, and entry points here.
    println!("EventFlow name: {}", evfl.name);
    
    // 3. Write it back to binary format
    let mut output = Cursor::new(Vec::new());
    evfl.write(&mut output);
    
    // 4. Save to a new file
    fs::write("example_out.bfevfl", output.into_inner()).expect("Failed to write file");
}
```

### Accessing Flowchart Data

```rust
use revfl::evfl::EventFlow;
use std::fs;

fn main() {
    let data = fs::read("example.bfevfl").unwrap();
    let mut evfl = EventFlow::new();
    evfl.read(&data);

    if let Some(flowchart) = &evfl.flowchart {
        println!("Flowchart Name: {}", flowchart.name);
        println!("Number of Actors: {}", flowchart.actors.len());
        println!("Number of Events: {}", flowchart.events.len());
        
        for entry_point in &flowchart.entry_points {
            println!("Entry Point: {}", entry_point.name);
        }
    }
}
```

## Structure

The core structures provided by this library reflect the BFEVFL format:
- `EventFlow`: The root container that holds either a `Flowchart` or a `Timeline` (or both).
- `Flowchart`: Contains logic elements like `Actor`s, `Event`s, and `EntryPoint`s.
- `Timeline`: Contains cutscene sequencing elements.
- `Actor`: Entities involved in the event flow, which hold `Action`s and `Query`s.
- `Event`: Represents nodes in the flow, such as actions, forks, joins, subflows, or switches.

## Acknowledgments

This library is a Rust port of the original Python implementation by [leoetlino](https://github.com/leoetlino).
The original `evfl` library can be found at: [https://github.com/zeldamods/evfl](https://github.com/zeldamods/evfl).
Ported to Rust by [carbonatedtea].

## License

This project is licensed under the MIT License.
