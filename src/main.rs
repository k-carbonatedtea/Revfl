use revfl::evfl::EventFlow;
use std::fs;
use std::path::Path;
use std::io::Cursor;

use std::env;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir = Path::new(manifest_dir).join("evfl/original");
    let mut success_count = 0;
    let mut total_count = 0;

    println!("Starting validation tests...");
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.extension().map(|s| s == "bfevfl" || s == "bfevtm").unwrap_or(false) {
            total_count += 1;
            let file_name = path.file_name().unwrap().to_string_lossy();
            println!("Testing {}", file_name);
            
            let original_data = fs::read(&path).unwrap();
            let mut evfl = EventFlow::new();
            
            // Read
            evfl.read(&original_data);
            
            // Write
            let mut output_data = Cursor::new(Vec::new());
            evfl.write(&mut output_data);
            
            let result_data = output_data.into_inner();
            
            if original_data == result_data {
                println!("  [OK] Matched exactly!");
                success_count += 1;
            } else {
                println!("  [FAIL] Did not match.");
                println!("    Original size: {}", original_data.len());
                println!("    Result size:   {}", result_data.len());
                // Find first mismatch
                let min_len = std::cmp::min(original_data.len(), result_data.len());
                for i in 0..min_len {
                    if original_data[i] != result_data[i] {
                        println!("    First mismatch at offset 0x{:04X}: expected 0x{:02X}, got 0x{:02X}", i, original_data[i], result_data[i]);
                        break;
                    }
                }
            }
        }
    }
    
    println!("\nTest Summary: {}/{} passed", success_count, total_count);
    if success_count != total_count {
        std::process::exit(1);
    }
}
