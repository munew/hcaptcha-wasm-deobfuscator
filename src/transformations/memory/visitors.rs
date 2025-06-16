use walrus::ir::{dfs_in_order, BinaryOp, Binop, Const, Load, Store, StoreKind, Value, Visitor};
use walrus::LocalFunction;
use crate::transformations::memory::MemEncFuncType;

#[derive(Default)]
pub struct LoadMemoryFuncMapper {
    has_load: bool,

    has_16_bits_mask: bool,
    has_8_bits_mask: bool,
    has_24_bits_shl: bool,
    has_right_shift_signed: bool,
    has_left_shift: bool,
}

impl LoadMemoryFuncMapper {
    pub fn map(&mut self, local: &LocalFunction) -> Option<MemEncFuncType> {
        dfs_in_order(self, local, local.entry_block());
        if !self.has_load {
            return None;
        }

        if self.has_left_shift && self.has_24_bits_shl {
            return Some(MemEncFuncType::Signed8);
        }

        if self.has_right_shift_signed {
            if self.has_16_bits_mask {
                return Some(MemEncFuncType::Signed16);
            } else if self.has_8_bits_mask {
                return Some(MemEncFuncType::Signed8);
            }
        } else {
            if self.has_16_bits_mask {
                return Some(MemEncFuncType::Unsigned16);
            } else if self.has_8_bits_mask {
                return Some(MemEncFuncType::Unsigned8);
            }
        }

        Some(MemEncFuncType::Signed32)
    }
}

impl<'a> Visitor<'a> for LoadMemoryFuncMapper {
    fn visit_const(&mut self, instr: &Const) {
        match instr.value {
            Value::I32(i) => {
                if i == 65535 {
                    self.has_16_bits_mask = true;
                }

                if i == 24 {
                    self.has_24_bits_shl = true;
                }

                if i == 255 {
                    self.has_8_bits_mask = true;
                }
            }
            _ => {}
        }
    }

    fn visit_binop(&mut self, instr: &Binop) {
        match instr.op {
            BinaryOp::I32ShrS => {
                self.has_right_shift_signed = true;
            }
            BinaryOp::I32Shl => {
                self.has_left_shift = true;
            }
            _ => {}
        }
    }

    fn visit_load(&mut self, _: &Load) {
        self.has_load = true;
    }
}

#[derive(Default)]
pub struct StoreMemoryFuncMapper {
    has_store: bool,
    store_kind: Option<StoreKind>,
}

impl StoreMemoryFuncMapper {
    pub fn map(&mut self, local: &LocalFunction) -> Option<MemEncFuncType> {
        dfs_in_order(self, local, local.entry_block());
        if !self.has_store {
            return None;
        }
        
        match self.store_kind.as_ref().unwrap() {
            StoreKind::I32_8 {..} => Some(MemEncFuncType::Signed8),
            StoreKind::I32_16 {..} => Some(MemEncFuncType::Signed16),
            StoreKind::I32 { .. } => Some(MemEncFuncType::Signed32),
            StoreKind::I64 { .. } => Some(MemEncFuncType::Float64),
            _ => unreachable!(),
        }
    }
}

impl<'a> Visitor<'a> for StoreMemoryFuncMapper {
    fn visit_store(&mut self, instr: &Store) {
        self.has_store = true;
        self.store_kind = Some(instr.kind.clone());
    }
}
