use crate::{AssemblerMessage, AssemblerMessageType, Flags};
use crate::nodes::*;
use crate::span::Span;
use crate::symbol_table::SymbolTable;
use crate::visitors::machine_code_generator::Rets::RawData;
use crate::visitors::Visitor;

pub struct MachineCodeGenerator<'a> {
    symbol_table: &'a SymbolTable,
    current_pos: u16,
    messages: Vec<AssemblerMessage>,
    flags: &'a Flags,
}

#[derive(Default, Debug)]
enum Rets {
    Raw(Vec<u8>),
    Instruction(u16),
    RawData(Vec<u8>),
    Reg(u8),
    Imm(u16),
    AddressImm(u16),
    #[default]
    Null,
}

impl<'a> Visitor<Rets, ()> for MachineCodeGenerator<'a> {
    fn visit_program(&mut self, node: &ProgramNode) -> Result<Rets, ()> {
        let mut instructions = Vec::new();
        instructions.resize(self.symbol_table.get_program_end_address() as usize, 0);

        if let Some(text) = &node.text_section {
            self.current_pos = self.symbol_table.get_text_section_base_address();

            let mut pos = self.current_pos;
            text.accept(self)?.get_raw_contents().iter().for_each(|b| {
                instructions[pos as usize] = *b;
                pos += 1;
            })
        }
        if let Some(data) = &node.data_section {
            self.current_pos = self.symbol_table.get_data_section_base_address();

            let mut pos = self.current_pos;
            data.accept(self)?.get_raw_contents().iter().for_each(|b| {
                instructions[pos as usize] = *b;
                pos += 1;
            })
        }

        Ok(Rets::Raw(instructions))
    }

    fn visit_data_section(&mut self, node: &DataSectionNode) -> Result<Rets, ()> {
        let mut statements = Vec::new();

        if self.flags.auto_align_sections && self.current_pos % 2 != 0 {
            self.current_pos += 1;
            statements.push(RawData(vec![0]));
        }

        for statement in &node.statements {
            statements.push(statement.accept(self)?);
        }

        Ok(Rets::Raw(
            statements.iter()
                .map(|r| {
                    match r {
                        Rets::Instruction(i) => {
                            self.add_warning("Found an instruction on .data", None);
                            Some(i.to_le_bytes().to_vec())
                        }
                        Rets::RawData(d) => {
                            Some(d.to_vec())
                        }
                        _ => None
                    }
                })
                .flatten()
                .flatten()
                .collect()
        ))
    }

    fn visit_text_section(&mut self, node: &TextSectionNode) -> Result<Rets, ()> {
        let mut statements = Vec::new();

        if self.flags.auto_align_sections && self.current_pos % 2 != 0 {
            self.current_pos += 1;
            statements.push(RawData(vec![0]));
        }

        for statement in &node.statements {
            statements.push(statement.accept(self)?);
        }

        Ok(Rets::Raw(
            statements.iter()
                .map(|r| {
                    match r {
                        Rets::Instruction(i) => {
                            Some(i.to_le_bytes().to_vec())
                        }
                        Rets::RawData(d) => {
                            self.add_warning("Found raw data in .text!", None);
                            Some(d.to_vec())
                        }
                        _ => None
                    }
                })
                .flatten()
                .flatten()
                .collect()
        ))
    }

    fn visit_statement(&mut self, node: &StatementNode) -> Result<Rets, ()> {
        match node {
            StatementNode::Instruction(i) => i.accept(self),
            StatementNode::RawData(r) => r.accept(self),
            _ => Ok(Default::default()),
        }
    }

    fn visit_instruction(&mut self, node: &InstructionNode) -> Result<Rets, ()> {
        let pc = self.current_pos;
        self.current_pos += 2;

        self.codify_instruction(node, pc)
            .map(|raw| Rets::Instruction(raw))
            .inspect_err(|e| self.add_error(e, None))
            .map_err(|_| ())
    }

    fn visit_raw_data(&mut self, node: &RawDataNode) -> Result<Rets, ()> {
        match node {
            RawDataNode::WordAlign => {
                if self.current_pos % 2 == 0 {
                    Ok(Rets::Null)
                } else {
                    self.current_pos += 1;
                    Ok(Rets::RawData(vec![0]))
                }
            }
            RawDataNode::Bytes(data) => {
                self.current_pos += node.get_size(self.current_pos);

                let mut bytes = Vec::new();
                for node in data {
                    bytes.push(node.accept(self)
                        .map_or(Ok(0), |r| r.as_u8())
                        .inspect_err(|e| self.add_error(e, None))
                    );
                }

                Ok(Rets::RawData(bytes.iter().flatten().map(|r| *r).collect()))
            }
            RawDataNode::Words(data) => {
                self.current_pos += node.get_size(self.current_pos);

                let mut bytes = Vec::new();

                if self.flags.auto_align_words && self.current_pos % 2 != 0 {
                    self.current_pos += 1;
                    bytes.push(0);
                }

                for node in data {
                    bytes.extend(node.accept(self)?.as_u16().to_le_bytes());
                }

                Ok(Rets::RawData(bytes))
            }
        }
    }

    fn visit_registry(&mut self, node: &RegistryNode) -> Result<Rets, ()> {
        Ok(Rets::Reg(node.reg))
    }

    fn visit_literal(&mut self, node: &LiteralNode) -> Result<Rets, ()> {
        match node {
            LiteralNode::Constant(c) => Ok(Rets::Imm(*c)),
            LiteralNode::SymbolRef(sr) => sr.accept(self),
            LiteralNode::Function(f) => f.accept(self),
        }
    }

    fn visit_symbol_ref(&mut self, node: &SymbolRefNode) -> Result<Rets, ()> {
        let symbol = match self.symbol_table.get_symbol(&node.name) {
            Ok(s) => s,
            Err(e) => {
                self.add_error(&e, None);
                return Ok(Rets::Imm(0));
            }
        };

        if symbol.is_address() {
            Ok(Rets::AddressImm(symbol.get_value()))
        } else {
            Ok(Rets::Imm(symbol.get_value()))
        }
    }

    fn visit_function(&mut self, node: &FunctionNode) -> Result<Rets, ()> {
        match node {
            FunctionNode::Lo(v) => Ok(Rets::Imm(v.accept(self)?.as_u16() & 0xFF)),
            FunctionNode::Hi(v) => Ok(Rets::Imm(v.accept(self)?.as_u16() >> 8)),
        }
    }
}

fn codify_3r(opcode: u8, ra: u8, rb: u8, rd: u8, f: u8) -> u16 {
    ((opcode as u16 & 0b1111) << 12)
        | ((ra as u16 & 0b111) << 9)
        | ((rb as u16 & 0b111) << 6)
        | ((rd as u16 & 0b111) << 3)
        | (f as u16 & 0b111)
}

fn codify_2r(opcode: u8, ra: u8, rb: u8, n: u8) -> u16 {
    ((opcode as u16 & 0b1111) << 12)
        | ((ra as u16 & 0b111) << 9)
        | ((rb as u16 & 0b111) << 6)
        | (n as u16 & 0b111111)
}

fn codify_1r(opcode: u8, ra: u8, ext: bool, n: u8) -> u16 {
    ((opcode as u16 & 0b1111) << 12)
        | ((ra as u16 & 0b111) << 9)
        | ((ext as u16 & 0b1) << 8)
        | (n as u16 & 0b11111111)
}

impl<'a> MachineCodeGenerator<'a> {
    pub fn generate(&mut self, node: &ProgramNode) -> Option<Vec<u8>> {
        node.accept(self).ok().map(|r| r.get_raw_contents())
    }

    pub fn new(symbol_table: &'a SymbolTable, flags: &'a Flags) -> Self {
        Self {
            symbol_table,
            current_pos: 0,
            messages: Vec::new(),
            flags,
        }
    }

    pub fn get_messages(&self) -> Vec<AssemblerMessage> {
        self.messages.clone()
    }

    fn add_warning(&mut self, message: &str, span: Option<Span>) {
        self.messages.push(AssemblerMessage {
            msg_type: AssemblerMessageType::WARNING,
            description: message.to_string(),
            span
        });
    }

    fn add_error(&mut self, message: &str, span: Option<Span>) {
        self.messages.push(AssemblerMessage {
            msg_type: AssemblerMessageType::ERROR,
            description: message.to_string(),
            span
        })
    }

    fn codify_instruction(&mut self, node: &InstructionNode, pc: u16) -> Result<u16, String> {
        Ok(match node {
            InstructionNode::And { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                0,
            ),
            InstructionNode::Or { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                1,
            ),
            InstructionNode::Xor { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                2,
            ),
            InstructionNode::Not { rd, ra } => {
                codify_3r(0x0, ra.accept(self).map_or(Ok(0), |r| r.as_u8())?, 0, rd.accept(self).map_or(Ok(0), |r| r.as_u8())?, 3)
            }
            InstructionNode::Add { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                4,
            ),
            InstructionNode::Sub { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                5,
            ),
            InstructionNode::Sha { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                6,
            ),
            InstructionNode::Shl { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                7,
            ),
            InstructionNode::Cmplt { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                0,
            ),
            InstructionNode::Cmple { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                1,
            ),
            InstructionNode::Cmpeq { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                3,
            ),
            InstructionNode::Cmpltu { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                4,
            ),
            InstructionNode::Cmpleu { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                5,
            ),
            InstructionNode::Addi { rd, ra, n6 } => codify_2r(
                0x2,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                n6.accept(self).map_or(Ok(0), |r| r.as_u8())?,
            ),
            InstructionNode::Ld { rd, n6, ra } => codify_2r(
                0x3,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                n6.accept(self).map_or(Ok(0), |r| r.as_u8())?,
            ),
            InstructionNode::St { n6, ra, rb } => codify_2r(
                0x4,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                n6.accept(self).map_or(Ok(0), |r| r.as_u8())?,
            ),
            InstructionNode::Ldb { rd, n6, ra } => codify_2r(
                0x5,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rd.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                n6.accept(self).map_or(Ok(0), |r| r.as_u8())?,
            ),
            InstructionNode::Stb { n6, ra, rb } => codify_2r(
                0x6,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                rb.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                n6.accept(self).map_or(Ok(0), |r| r.as_u8())?,
            ),
            InstructionNode::Jalr { rd, ra } => {
                codify_2r(0x7, ra.accept(self).map_or(Ok(0), |r| r.as_u8())?, rd.accept(self).map_or(Ok(0), |r| r.as_u8())?, 0)
            }
            InstructionNode::Bz { ra, n8 } => codify_1r(
                0x8,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                false,
                n8.accept(self).map_or(Ok(0), |r| r.as_u8_relative(pc + 2))?,
            ),
            InstructionNode::Bnz { ra, n8 } => codify_1r(
                0x8,
                ra.accept(self).map_or(Ok(0), |r| r.as_u8())?,
                true,
                n8.accept(self).map_or(Ok(0), |r| r.as_u8_relative(pc + 2))?,
            ),
            InstructionNode::Movi { rd, n8 } => {
                codify_1r(0x9, rd.accept(self).map_or(Ok(0), |r| r.as_u8())?, false, n8.accept(self).map_or(Ok(0), |r| r.as_u8())?)
            }
            InstructionNode::Movhi { rd, n8 } => {
                codify_1r(0x9, rd.accept(self).map_or(Ok(0), |r| r.as_u8())?, true, n8.accept(self).map_or(Ok(0), |r| r.as_u8())?)
            }
            InstructionNode::In { rd, n8 } => {
                codify_1r(0xA, rd.accept(self).map_or(Ok(0), |r| r.as_u8())?, false, n8.accept(self).map_or(Ok(0), |r| r.as_u8())?)
            }
            InstructionNode::Out { n8, ra } => {
                codify_1r(0xA, ra.accept(self).map_or(Ok(0), |r| r.as_u8())?, true, n8.accept(self).map_or(Ok(0), |r| r.as_u8())?)
            }
        })
    }
}

impl Rets {
    fn as_u8(&self) -> Result<u8, String> {
        match self {
            Rets::Reg(r) => Ok(*r),
            Rets::Imm(i) => as_u8_lossless(*i),
            Rets::AddressImm(_) => Err(String::from("Can't fit an address in a byte!")),
            Rets::Null => panic!("Attempted to read a NULL value as a u8"),
            x => panic!("Called Rets::as_u8() on an invalid value: {:?}", x),
        }
    }

    fn as_u16(&self) -> u16 {
        match self {
            Rets::Reg(r) => *r as u16,
            Rets::Imm(i) => *i,
            Rets::AddressImm(i) => *i,
            Rets::Null => panic!("Attempted to read a NULL value as a u16"),
            x => panic!("Called Rets::as_u16() on an invalid value: {:?}", x),
        }
    }

    fn as_u8_relative(&self, rel_to: u16) -> Result<u8, String> {
        match self {
            Rets::Imm(i) => as_u8_lossless(*i),
            Rets::AddressImm(li) => Ok((*li).wrapping_sub(rel_to).wrapping_div(2) as u8),
            x => panic!("Called Rets::as_u8_relative() on an invalid value: {:?}", x),
        }
    }

    fn get_raw_contents(self) -> Vec<u8> {
        match self {
            Rets::Raw(i) => i,
            x => panic!("Called Rets::get_raw_contents() on an invalid value: {:?}", x),
        }
    }
}

fn as_u8_lossless(val: u16) -> Result<u8, String> {
    if val <= 255 || (val as i16) == (val as i8) as i16 {
        Ok(val as u8)
    } else {
        Err(String::from("This value doesn't fit in a byte!"))
    }
}