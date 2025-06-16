mod transformations;
mod fetcher;

use std::time::Instant;
use crate::transformations::Transformer;
use walrus::Module;
use crate::fetcher::events::fetch_events;
use crate::transformations::memory::memory_transformer::MemoryTransformer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let t = Instant::now();
    let mut module = Module::from_file("./assets/vm_input.wasm").map_err(|e| format!("{:?}", e))?;
    
    let mut transformers = vec![
        MemoryTransformer{},
    ];
    
    for transformer in transformers.iter_mut() {
        transformer.transform(&mut module);
    }
    
    let events = fetch_events(&mut module)?;
    println!("{:?}", events);
    
    println!("Took {:?}", t.elapsed());
    module.emit_wasm_file("assets/output.wasm")?;
    
    Ok(())
}
