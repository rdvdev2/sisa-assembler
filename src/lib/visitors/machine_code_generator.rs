use crate::nodes::_node_traits::NodeVisitor as nvst;
use crate::nodes::*;
use crate::span::Span;
use crate::symbol_table::SymbolTable;
use crate::{AssemblerMessage, AssemblerMessageType, Flags};
use easy_nodes::Node;

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

impl<'a> NodeVisitor<Rets> for MachineCodeGenerator<'a> {
    fn visit_program(&mut self, _span: &Span, program: &Program) -> Rets {
        let mut instructions = Vec::new();
        instructions.resize(self.symbol_table.get_program_end_address() as usize, 0);

        if let Some(text) = &program.text_section {
            self.current_pos = self.symbol_table.get_text_section_base_address();

            let mut pos = self.current_pos;
            text.accept(self).get_raw_contents().iter().for_each(|b| {
                instructions[pos as usize] = *b;
                pos += 1;
            })
        }
        if let Some(data) = &program.data_section {
            self.current_pos = self.symbol_table.get_data_section_base_address();

            let mut pos = self.current_pos;
            data.accept(self).get_raw_contents().iter().for_each(|b| {
                instructions[pos as usize] = *b;
                pos += 1;
            })
        }

        Rets::Raw(instructions)
    }

    fn visit_data_section(&mut self, _span: &Span, ds: &DataSection) -> Rets {
        let mut statements = Vec::new();

        if self.flags.auto_align_sections && self.current_pos % 2 != 0 {
            self.current_pos += 1;
            statements.push(Rets::RawData(vec![0]));
        }

        for statement in &ds.statements {
            statements.push(statement.accept(self));
        }

        Rets::Raw(
            statements
                .iter()
                .map(|r| match r {
                    Rets::Instruction(i) => {
                        self.add_warning("Found an instruction on .data", None);
                        Some(i.to_le_bytes().to_vec())
                    }
                    Rets::RawData(d) => Some(d.to_vec()),
                    _ => None,
                })
                .flatten()
                .flatten()
                .collect(),
        )
    }

    fn visit_text_section(&mut self, _span: &Span, ts: &TextSection) -> Rets {
        let mut statements = Vec::new();

        if self.flags.auto_align_sections && self.current_pos % 2 != 0 {
            self.current_pos += 1;
            statements.push(Rets::RawData(vec![0]));
        }

        for statement in &ts.statements {
            statements.push(statement.accept(self));
        }

        Rets::Raw(
            statements
                .iter()
                .map(|r| match r {
                    Rets::Instruction(i) => Some(i.to_le_bytes().to_vec()),
                    Rets::RawData(d) => {
                        self.add_warning("Found raw data in .text!", None);
                        Some(d.to_vec())
                    }
                    _ => None,
                })
                .flatten()
                .flatten()
                .collect(),
        )
    }

    fn visit_statement(&mut self, _span: &Span, statement: &Statement) -> Rets {
        match statement {
            Statement::Instruction(i) => i.accept(self),
            Statement::RawData(r) => r.accept(self),
            _ => Default::default(),
        }
    }

    fn visit_instruction(&mut self, span: &Span, instruction: &Instruction) -> Rets {
        let pc = self.current_pos;
        self.current_pos += 2;

        self.codify_instruction(instruction, pc)
            .map(|raw| Rets::Instruction(raw))
            .inspect_err(|e| self.add_error(e, Some(*span)))
            .unwrap_or(Default::default())
    }

    fn visit_raw_data(&mut self, span: &Span, raw_data: &RawData) -> Rets {
        match raw_data {
            RawData::WordAlign => {
                if self.current_pos % 2 == 0 {
                    Rets::Null
                } else {
                    self.current_pos += 1;
                    Rets::RawData(vec![0])
                }
            }
            RawData::Bytes(data) => {
                self.current_pos += raw_data.get_size(self.current_pos);

                let mut bytes = Vec::new();
                for node in data {
                    bytes.push(
                        node.accept(self)
                            .as_u8()
                            .inspect_err(|e| self.add_error(e, Some(*span))),
                    );
                }

                Rets::RawData(bytes.iter().flatten().map(|r| *r).collect())
            }
            RawData::Words(data) => {
                self.current_pos += raw_data.get_size(self.current_pos);

                let mut bytes = Vec::new();

                if self.flags.auto_align_words && self.current_pos % 2 != 0 {
                    self.current_pos += 1;
                    bytes.push(0);
                }

                for node in data {
                    bytes.extend(node.accept(self).as_u16().to_le_bytes());
                }

                Rets::RawData(bytes)
            }
        }
    }

    fn visit_registry(&mut self, _span: &Span, registry: &Registry) -> Rets {
        Rets::Reg(registry.reg)
    }

    fn visit_literal(&mut self, _span: &Span, literal: &Literal) -> Rets {
        match literal {
            Literal::Constant(c) => Rets::Imm(*c),
            Literal::SymbolRef(sr) => sr.accept(self),
            Literal::Function(f) => f.accept(self),
        }
    }

    fn visit_symbol_ref(&mut self, span: &Span, symbol_ref: &SymbolRef) -> Rets {
        let symbol = match self.symbol_table.get_symbol(&symbol_ref.name) {
            Ok(s) => s,
            Err(e) => {
                self.add_error(&e, Some(*span));
                return Rets::Imm(0);
            }
        };

        if symbol.is_address() {
            Rets::AddressImm(symbol.get_value())
        } else {
            Rets::Imm(symbol.get_value())
        }
    }

    fn visit_function(&mut self, _span: &Span, function: &Function) -> Rets {
        match function {
            Function::Lo(v) => Rets::Imm(v.accept(self).as_u16() & 0xFF),
            Function::Hi(v) => Rets::Imm(v.accept(self).as_u16() >> 8),
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
    pub fn generate(&mut self, node: &Node<Span, Program>) -> Option<Vec<u8>> {
        Some(node.accept(self).get_raw_contents())
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
            span,
        });
    }

    fn add_error(&mut self, message: &str, span: Option<Span>) {
        self.messages.push(AssemblerMessage {
            msg_type: AssemblerMessageType::ERROR,
            description: message.to_string(),
            span,
        })
    }

    fn codify_instruction(&mut self, node: &Instruction, pc: u16) -> Result<u16, String> {
        Ok(match node {
            Instruction::And { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                0,
            ),
            Instruction::Or { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                1,
            ),
            Instruction::Xor { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                2,
            ),
            Instruction::Not { rd, ra } => codify_3r(
                0x0,
                ra.accept(self).as_u8()?,
                0,
                rd.accept(self).as_u8()?,
                3,
            ),
            Instruction::Add { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                4,
            ),
            Instruction::Sub { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                5,
            ),
            Instruction::Sha { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                6,
            ),
            Instruction::Shl { rd, ra, rb } => codify_3r(
                0x0,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                7,
            ),
            Instruction::Cmplt { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                0,
            ),
            Instruction::Cmple { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                1,
            ),
            Instruction::Cmpeq { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                3,
            ),
            Instruction::Cmpltu { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                4,
            ),
            Instruction::Cmpleu { rd, ra, rb } => codify_3r(
                0x1,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                5,
            ),
            Instruction::Addi { rd, ra, n6 } => codify_2r(
                0x2,
                ra.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                n6.accept(self).as_u8()?,
            ),
            Instruction::Ld { rd, n6, ra } => codify_2r(
                0x3,
                ra.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                n6.accept(self).as_u8()?,
            ),
            Instruction::St { n6, ra, rb } => codify_2r(
                0x4,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                n6.accept(self).as_u8()?,
            ),
            Instruction::Ldb { rd, n6, ra } => codify_2r(
                0x5,
                ra.accept(self).as_u8()?,
                rd.accept(self).as_u8()?,
                n6.accept(self).as_u8()?,
            ),
            Instruction::Stb { n6, ra, rb } => codify_2r(
                0x6,
                ra.accept(self).as_u8()?,
                rb.accept(self).as_u8()?,
                n6.accept(self).as_u8()?,
            ),
            Instruction::Jalr { rd, ra } => {
                codify_2r(0x7, ra.accept(self).as_u8()?, rd.accept(self).as_u8()?, 0)
            }
            Instruction::Bz { ra, n8 } => codify_1r(
                0x8,
                ra.accept(self).as_u8()?,
                false,
                n8.accept(self).as_u8_relative(pc + 2)?,
            ),
            Instruction::Bnz { ra, n8 } => codify_1r(
                0x8,
                ra.accept(self).as_u8()?,
                true,
                n8.accept(self).as_u8_relative(pc + 2)?,
            ),
            Instruction::Movi { rd, n8 } => codify_1r(
                0x9,
                rd.accept(self).as_u8()?,
                false,
                n8.accept(self).as_u8()?,
            ),
            Instruction::Movhi { rd, n8 } => codify_1r(
                0x9,
                rd.accept(self).as_u8()?,
                true,
                n8.accept(self).as_u8()?,
            ),
            Instruction::In { rd, n8 } => codify_1r(
                0xA,
                rd.accept(self).as_u8()?,
                false,
                n8.accept(self).as_u8()?,
            ),
            Instruction::Out { n8, ra } => codify_1r(
                0xA,
                ra.accept(self).as_u8()?,
                true,
                n8.accept(self).as_u8()?,
            ),
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
            x => panic!(
                "Called Rets::get_raw_contents() on an invalid value: {:?}",
                x
            ),
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
