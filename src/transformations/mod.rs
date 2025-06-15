pub mod memory;

use walrus::Module;

pub trait Transformer {
    fn transform(&mut self, module: &mut Module);
}