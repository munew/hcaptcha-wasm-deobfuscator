use walrus::ir::{dfs_in_order, Const, Value, Visitor};
use walrus::LocalFunction;

struct I32ConstCollector {
    collected_consts: Vec<i32>,
}

impl<'a> Visitor<'a> for I32ConstCollector {
    fn visit_const(&mut self, instr: &Const) {
        if let Value::I32(i) = &instr.value {
            self.collected_consts.push(*i);
        }
    }
}

pub fn collect_i32_consts(local: &LocalFunction) -> Vec<i32> {
    let mut visitor = I32ConstCollector { collected_consts: vec![] };
    dfs_in_order(&mut visitor, local, local.entry_block());
    visitor.collected_consts
}