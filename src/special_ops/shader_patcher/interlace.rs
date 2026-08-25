use crate::log;

use super::hash;
use d3dasm;
use dxbc_ir_parser::build::{self as ir,  dcl_input_ps_siv, discard_z,  ftou};
use dxbc::{self, ChunkData, build_dxbc, chunks::WritableChunk, shex::{Program, encode}
};

pub fn interlace_patch(bytecode: &[u8]) -> Vec<u8> {
    let container_vec = d3dasm::parse(&bytecode);
    let Some(shader) = container_vec.get(0) else {
        log!("False shader sent over!");
        return bytecode.to_vec();
    };
    
    let mut modded_chunks: Vec<WritableChunk> = vec![];
    let position_reg:Option<u32> = (||{
        let isgn = shader.input_signature()?;
        let element = isgn.elements.iter().find(|el|
        el.semantic_name.as_bytes().eq_ignore_ascii_case(b"SV_POSITION"))?;
        Some(element.register)
    })();

    if position_reg.is_none() {
        log!("No-position-PS detected!");
        return bytecode.to_vec();
    }
    for chunk in shader.chunks() {
        match chunk {
            ChunkData::Shader(program) => {
                modded_chunks.push(WritableChunk { 
                fourcc: program.fourcc, 
                data: encode(&modify_program(&program, position_reg.unwrap())) 
            });
            }

            _=> {
                modded_chunks.push(chunk.to_writable().unwrap());
            }
        }
    }

    let mut modded_bytecode = build_dxbc(&modded_chunks);
    hash::patch_dxbc_checksum(&mut modded_bytecode);
    modded_bytecode

}



fn modify_program(program: &Program, position_reg: u32) -> Program{
    let mut new_inst = vec![
        ir::dcl_input_ps_siv("linear_noperspective", ir::input(position_reg).xy(), "position"),
        ir::ftoi(ir::temp(0).y_mask(), ir::input(0).scalar(1)),
        ir::and(ir::temp(0).x_mask(), ir::temp(0).scalar(1), ir::imm_i32x1(1)),
        ir::discard_nz(ir::temp(0).scalar(0))
        ];
    let mut instructions = program.instructions.clone();
    for (i, inst) in instructions.clone().iter().enumerate() {
        let inst_str = inst.to_string();
        if inst_str.contains("dcl_input_ps") && inst_str.contains("position") {
            new_inst.remove(0);
        }
        if !inst_str.contains("dcl_") {
            instructions.splice(i..i, new_inst);
            break;
        }
    }
    Program{
        instructions: instructions,
        ..program.clone()
    }
}