use crate::assembler::message::{AssemblerMessage, AssemblerMessageType};
use crate::{Flags, Lexer, MachineCodeGenerator, Parser, SymbolTableBuilder};

pub mod message;

pub struct Assembler<'a> {
    flags: &'a Flags,
}

pub struct AssemblerResult {
    pub machine_code: Option<Vec<u8>>,
    pub assembler_messages: Vec<AssemblerMessage>,
}

impl<'a> Assembler<'a> {
    pub fn new(flags: &'a Flags) -> Self {
        Self { flags }
    }

    pub fn assemble(&self, code: &str) -> AssemblerResult {
        let mut result = AssemblerResult {
            machine_code: None,
            assembler_messages: Vec::new(),
        };

        let lexer = Lexer::new(code);
        let parser = Parser::new(lexer);
        let node = match parser.parse() {
            Ok(n) => n,
            Err(e) => {
                result.assembler_messages.push(e);
                return result;
            }
        };

        let mut symbol_table_builder = SymbolTableBuilder::new(self.flags);
        symbol_table_builder.build(&node);
        result
            .assembler_messages
            .extend(symbol_table_builder.get_messages());
        let symbol_table = symbol_table_builder.get_symbol_table();

        let mut machine_code_generator = MachineCodeGenerator::new(&symbol_table, self.flags);
        result.machine_code = machine_code_generator.generate(&node);
        result
            .assembler_messages
            .extend(machine_code_generator.get_messages());

        if result
            .assembler_messages
            .iter()
            .any(|msg| msg.msg_type == AssemblerMessageType::ERROR)
        {
            result.machine_code = None;
        }

        result
    }
}
