mod visitor;

use crate::fetcher::events::visitor::collect_i32_consts;
use anyhow::{bail, Context};
use std::collections::VecDeque;
use walrus::ir::{BinaryOp, Block, Const, IfElse, Instr, Loop, Value};
use walrus::{ConstExpr, DataKind, GlobalKind, LocalFunction, Module};

const NEEDED_VALUES: [i32; 4] = [-1, 268435455, -2147483648, 0]; 

pub fn fetch_events(module: &mut Module) -> Result<String, anyhow::Error> {
    let global = module.globals.iter().next().context("Could not find global")?;
    let data_segment = module.data.iter().nth(1).context("Could not find memory")?;
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
    
    
    'a: for (_, func) in module.funcs.iter_local() {
        let collected_consts = collect_i32_consts(func);
        for n in NEEDED_VALUES {
            if !collected_consts.contains(&n) {
                continue 'a;
            }
        }
        
        let (events_idx, events_length) = search_pattern(data_start, func).context("Could not find xor event loc in memory")?;
        let global_idx = match &global.kind {
            GlobalKind::Local(c) => match c {
                ConstExpr::Value(v) => match v {
                    Value::I32(i) => i,
                    _ => panic!(),
                }
                _ => panic!(),
            }
            _ => panic!(),
        };
        
        return read_events(data_start, &data_segment.value, events_idx as usize, events_length as usize, *global_idx as usize);
    }
    
    bail!("could not find function that init events")
}

fn read_events(data_start: usize, data: &Vec<u8>, encrypted_event_string_idx: usize, events_length: usize, xor_table: usize) -> Result<String, anyhow::Error> {
    let mut res = String::new();
    let mut offset = 0;
    
    let off1 = encrypted_event_string_idx - data_start;
    let off2 = xor_table - data_start;
    
    while offset < (events_length+4) {
        let a = u8s_to_u32_le(data[off1+offset], data[off1+offset+1], data[off1+offset+2], data[off1+offset+3]);
        let b = u8s_to_u32_le(data[off2+offset], data[off2+offset+1], data[off2+offset+2], data[off2+offset+3]);
        let xor = a ^ b;
        
        res += &u32_to_string_le(xor);
        offset += 4;
    }
    
    Ok(res)
}

fn u8s_to_u32_le(a: u8, b: u8, c: u8, d: u8) -> u32 {
    ((d as u32) << 24) | ((c as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

fn u32_to_string_le(value: u32) -> String {
    let bytes = value.to_le_bytes();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn search_pattern(data_segment_start: usize, func: &LocalFunction) -> Option<(i32, i32)> {
    let mut stack = VecDeque::new();
    stack.push_front(func.entry_block());
    while let Some(block_id) = stack.pop_back() {
        let block = func.block(block_id);

        for (idx, (instr, _)) in block.instrs.iter().enumerate() {
            match instr {
                Instr::Block(Block { seq }) | Instr::Loop(Loop { seq }) => {
                    stack.push_front(*seq)
                }
                Instr::IfElse(IfElse {
                                  consequent,
                                  alternative,
                              }) => {
                    stack.push_front(*consequent);
                    stack.push_front(*alternative);
                }
                Instr::Binop(op) if matches!(op.op, BinaryOp::I32Xor) => {
                    if !matches!(block.instrs[idx-1].0, Instr::Load(_)) || !matches!(block.instrs[idx-2].0, Instr::Binop(_)) || !matches!(block.instrs[idx+1].0, Instr::Store(_)) {
                        continue;
                    }


                    if let (
                        Some(Instr::Const(Const { value: Value::I32(n1), .. })),
                        Some(Instr::Const(Const { value: Value::I32(n2), .. })),
                    ) = (
                        block.instrs.get(idx.wrapping_sub(3)).map(|(i, _)| i),
                        block.instrs.get(idx + 3).map(|(i, _)| i),
                    ) {
                        if *n1 > data_segment_start as i32 {
                            return Some((*n1, *n2));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    
    None
}