#![feature(proc_macro_hygiene)]
#![feature(derive_default_enum)]
#![feature(assert_matches)]
#![feature(result_option_inspect)]
#![feature(int_abs_diff)]
#![feature(try_trait_v2)]

extern crate core;

mod assembler;
mod lexer;
mod nodes;
mod parser;
mod span;
mod symbol_table;
mod tokens;
mod visitors;
mod flags;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::visitors::{MachineCodeGenerator, SymbolTableBuilder};
use colorful::Colorful;
use std::io::{Read, Write};
use std::{fs, path};

pub use flags::*;
use crate::assembler::Assembler;
use crate::assembler::message::{AssemblerMessage, AssemblerMessageType};
use crate::span::Span;

pub fn assemble(source_file: &path::Path, output_file: &path::Path, flags: Flags) -> Result<String, String> {
    let code = read_source(source_file)?;

    let assembler = Assembler::new(flags);
    let asm_result = assembler.assemble(&code);

    if let Some(machine_code) = asm_result.machine_code {
        write_output(output_file, machine_code.as_slice())?;
        Ok(write_messages(asm_result.assembler_messages, source_file, &code))
    } else {
        Err(write_messages(asm_result.assembler_messages, source_file, &code))
    }
}


fn read_source(path: &path::Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("Error opening source file: {}", e))?;

    let mut code = String::new();
    file.read_to_string(&mut code)
        .map_err(|e| format!("Error reading source file: {}", e))?;
    code += "\n";
    Ok(code)
}

fn write_output(path: &path::Path, data: &[u8]) -> Result<(), String> {
    let mut file =
        fs::File::create(path).map_err(|e| format!("Error creating output file: {}", e))?;

    file.write_all(data)
        .map_err(|e| format!("Error writing output file: {}", e))
}

fn write_messages(messages: Vec<AssemblerMessage>, source_path: &path::Path, code: &str) -> String {
    let mut ret = String::new();

    for msg in messages {
        ret += format!("{}\n", msg.write(source_path, code)).as_str();
    }

    ret
}