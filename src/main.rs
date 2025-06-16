mod fetcher;
mod transformations;

use crate::fetcher::events::fetch_events;
use crate::transformations::memory::memory_transformer::MemoryTransformer;
use crate::transformations::Transformer;
use std::env;
use std::path::Path;
use std::time::Instant;
use walrus::Module;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let input = Path::new(
        args.get(1)
            .map(|s| s.as_str())
            .unwrap_or("./assets/input.wasm"),
    );
    
    let output = Path::new(
        args.get(2)
            .map(|s| s.as_str())
            .unwrap_or("./assets/output.wasm"),
    );
    
    if !input.exists() {
        panic!("Input file does not exist");
    }

    let t = Instant::now();
    let mut module = Module::from_file(input).map_err(|e| format!("{:?}", e))?;

    let mut transformers = vec![MemoryTransformer {}];

    for transformer in transformers.iter_mut() {
        transformer.transform(&mut module);
    }

    let events = fetch_events(&mut module)?;
    println!("{:?}", events);

    println!("Took {:?}", t.elapsed());
    module.emit_wasm_file(output)?;

    Ok(())
}
