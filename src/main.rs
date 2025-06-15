mod transformations;
mod fetcher;

use std::time::Instant;
use std::collections::HashMap;
use crate::transformations::Transformer;
use walrus::Module;
use crate::fetcher::events::fetch_events;
use crate::transformations::memory::memory_transformer::MemoryTransformer;
use std::fs::File;
use std::io::Write;
use serde_json;

#[derive(Debug, serde::Serialize)]
struct Event {
    id: u32,
    hash: bool
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t = Instant::now();
    let mut module = Module::from_file("./assets/input.wasm").map_err(|e| format!("{:?}", e))?;
    
    let mut transformers = vec![
        MemoryTransformer{},
    ];
    
    for transformer in transformers.iter_mut() {
        transformer.transform(&mut module);
    }
    
    let events = fetch_events(&mut module)?;
    let mut events_map: HashMap<String, Event> = HashMap::new();
    
    for line in events.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 3 {
            let idx = parts[0].to_string();
            let id = u32::from_str_radix(parts[1], 16)?;
            let hash = parts[2] == "1";
            
            events_map.insert(idx, Event { id, hash });
        }
    }
    
    let json = serde_json::to_string_pretty(&events_map)?;
    let mut file = File::create("assets/events.json")?;
    file.write_all(json.as_bytes())?;
    
    println!("Took {:?}", t.elapsed());
    module.emit_wasm_file("assets/output.wasm")?;
    
    Ok(())
}
