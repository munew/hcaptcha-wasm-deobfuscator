mod visitor;

use crate::fetcher::events::visitor::collect_i32_consts;
use anyhow::{bail, Context};
use std::collections::VecDeque;
use std::ops::Index;
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
        
        return read_events(data_start, &data_segment.value, events_idx as usize, *global_idx as usize);
    }
    
    bail!("could not find function that init events")
}

fn read_events(data_start: usize, data: &Vec<u8>, encrypted_event_string_idx: usize, xor_table: usize) -> Result<String, anyhow::Error> {
    let mut res = String::new();
    let mut offset = 0;
    
    let off1 = encrypted_event_string_idx - data_start;
    let off2 = xor_table - data_start;

    // This is definitely not how hCaptcha does it
    // I'm just way too lazy to parse the length due to how hard it would be with control flow
    loop {
        let a = data[off1+offset];
        let b = data[off2+offset];
        let xor = (a ^ b) as char;
        
        if !xor.is_alphanumeric() && xor != '\n' && xor != ',' {
            break;
        }
        
        res.push(xor);
        offset += 1;
    }

    Ok(res)
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