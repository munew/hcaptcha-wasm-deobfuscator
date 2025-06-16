use std::collections::{HashMap, VecDeque};
use anyhow::Context;
use walrus::{ConstExpr, DataKind, FunctionId, Module};
use walrus::ir::{BinaryOp, Instr, Value};
use crate::transformations::memory::MemEncFuncType;

pub struct XorMemoryEncryption {
    xor_table_start: usize,
}

impl XorMemoryEncryption {
    fn decrypt(&self, module: &Module, start: usize, data: &Vec<u8>) -> (usize, Vec<u8>) {
        let start_pos = start - ((start / 320) << 3) - 320 - 23; // 23 is hardcoded btw
        let mut new_data = Vec::<u8>::with_capacity(data.len());

        let xor_table = self.get_xor_table(module);
        for (i, _) in data.iter().enumerate() {
            let pos = start_pos + i;

            let res = self.read_byte(start, &data, &xor_table, pos);
            if let Some(res) = res {
                new_data.push(res);
            } else {
                break;
            }
        }

        (start_pos, new_data)
    }
}

impl XorMemoryEncryption {
    // Retrieves xor table
    // Needs a function that loads a primitive from the memory (preferably unsigned byte)
    fn get_xor_table(&self, module: &Module) -> Vec<u8> {
        let mut xors = Vec::new();

        let data_segment = module.data.iter().next().unwrap();
        let data_start = match &data_segment.kind {
            DataKind::Active { offset, .. } => match *offset {
                ConstExpr::Value(v) => match v {
                    Value::I32(i) => i,
                    _ => panic!(),
                },
                _ => panic!(),
            },
            _ => panic!(),
        } as usize;

        // It seems like that the table always has 96 bytes
        for i in 0..96 {
            xors.push(data_segment.value[self.xor_table_start + i - data_start]);
        }

        xors
    }

    fn read_byte(
        &self,
        data_start: usize,
        data: &Vec<u8>,
        xor_table: &Vec<u8>,
        pos: usize,
    ) -> Option<u8> {
        let var0 = pos;
        let var1 = var0 / 320;
        let var2 = (var1 << 3) + var0 + 1032;

        let v = xor_table[var0 % 96];
        let result = if *data.get((var1 * 328 + 1024) - data_start)? > 0 {
            data[var2 - data_start]
        } else {
            v
        };

        Some(result ^ v)
    }
}

pub enum MemoryEncryptionMode {
    Xor(XorMemoryEncryption),
    Chacha20,
}

impl MemoryEncryptionMode {
    pub fn decrypt(&self, module: &Module, start: usize, data: &Vec<u8>) -> (usize, Vec<u8>) {
        match self {
            MemoryEncryptionMode::Xor(enc) => enc.decrypt(module, start, data),
            MemoryEncryptionMode::Chacha20 => panic!("Chacha20 is not supported yet"),
        }
    }
}

pub fn map_memory_encryption_mode(module: &Module, mapped_loads: &HashMap<FunctionId, MemEncFuncType>) -> Result<MemoryEncryptionMode, anyhow::Error> {
    let u8_load_func = mapped_loads
        .into_iter()
        .find(|(_, func_type)| matches!(func_type, MemEncFuncType::Unsigned8))
        .map(|(id, _)| module.funcs.get(*id).kind.unwrap_local())
        .context("could not find u8 load func")?;

    let mut stack = VecDeque::new();
    stack.push_front(u8_load_func.entry_block());

    while let Some(block_id) = stack.pop_back() {
        let block = u8_load_func.block(block_id);

        for window in block.instrs.windows(2).enumerate() {
            let (_, instrs) = window;

            match &instrs[0].0 {
                Instr::Call(_) => {
                    return Ok(MemoryEncryptionMode::Chacha20);
                }
                Instr::Binop(binop) if matches!(binop.op, BinaryOp::I32RemU) => {
                    if let Instr::Const(c) = &instrs[1].0 {
                        if let Value::I32(i) = c.value {
                            return Ok(MemoryEncryptionMode::Xor(XorMemoryEncryption {
                                xor_table_start: i as usize,
                            }));
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    Err(anyhow::anyhow!("Failed to map memory encryption mode"))
}