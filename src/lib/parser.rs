use crate::nodes::*;
use crate::span::Span;
use crate::tokens::Token::{self, *};
use plex::parser;
use crate::assembler::message::{AssemblerMessage, AssemblerMessageType};

pub struct Parser<T: Iterator<Item = (Token, Span)>> {
    tokens: T,
}

#[allow(unused_braces)] // parser! {} generates a lot of those warnings
impl<T: Iterator<Item = (Token, Span)>> Parser<T> {
    parser! {
        fn _parse(Token, Span);

        (a, b) {
            Span {
                lo: a.lo,
                hi: b.hi
            }
        }

        program: ProgramNode {
            data_section[ds] END => {
                let mut p = ProgramNode::empty();
                p.data_section = Some(ds);
                p
            }
            text_section[ts] END => {
                let mut p = ProgramNode::empty();
                p.text_section = Some(ts);
                p
            }
            data_section[ds] text_section[ts] END => {
                let mut p = ProgramNode::empty();
                p.data_section = Some(ds);
                p.text_section = Some(ts);
                p
            }
            text_section[ts] data_section[ds] END => {
                let mut p = ProgramNode::empty();
                p.data_section = Some(ds);
                p.text_section = Some(ts);
                p
            }
            constant[c] program[mut p] => {
                p.constants.push(c);
                p
            }
        }

        data_section: DataSectionNode {
            BEGIN_DATA => DataSectionNode::empty(),
            data_section[mut ds] statement[s] => {
                ds.statements.push(s);
                ds
            }
        }

        text_section: TextSectionNode {
            BEGIN_TEXT => TextSectionNode::empty(),
            text_section[mut ts] statement[s] => {
                ts.statements.push(s);
                ts
            }
        }

        statement: StatementNode {
            instruction[i] => StatementNode::Instruction(i),
            label[l] => StatementNode::Label(l),
            raw_data[r] => StatementNode::RawData(r),
            constant[c] => StatementNode::Constant(c),
        }

        instruction: InstructionNode {
            AND reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::And { rd, ra, rb },
            OR  reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Or  { rd, ra, rb },
            XOR reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Xor { rd, ra, rb },
            NOT reg[rd] COMMA reg[ra]               => InstructionNode::Not { rd, ra },
            ADD reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Add { rd, ra, rb },
            SUB reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Sub { rd, ra, rb },
            SHA reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Sha { rd, ra, rb },
            SHL reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Shl { rd, ra, rb },

            CMPLT  reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Cmplt  { rd, ra, rb },
            CMPLE  reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Cmple  { rd, ra, rb },
            CMPEQ  reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Cmpeq  { rd, ra, rb },
            CMPLTU reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Cmpltu { rd, ra, rb },
            CMPLEU reg[rd] COMMA reg[ra] COMMA reg[rb] => InstructionNode::Cmpleu { rd, ra, rb },

            ADDI reg[rd] COMMA reg[ra] COMMA lit[n6] => InstructionNode::Addi { rd, ra, n6 },

            LD  reg[rd] COMMA lit[n6] LPAR reg[ra] RPAR => InstructionNode::Ld  { rd, n6, ra },
            LDB reg[rd] COMMA lit[n6] LPAR reg[ra] RPAR => InstructionNode::Ldb { rd, n6, ra },
            ST  lit[n6] LPAR reg[ra] RPAR COMMA reg[rb] => InstructionNode::St  { n6, ra, rb },
            STB lit[n6] LPAR reg[ra] RPAR COMMA reg[rb] => InstructionNode::Stb { n6, ra, rb },

            JALR reg[rd] COMMA reg[ra] => InstructionNode::Jalr { rd, ra },
            BZ   reg[ra] COMMA lit[n8] => InstructionNode::Bz   { ra, n8 },
            BNZ  reg[ra] COMMA lit[n8] => InstructionNode::Bnz  { ra, n8 },

            MOVI  reg[rd] COMMA lit[n8] => InstructionNode::Movi  { rd, n8 },
            MOVHI reg[rd] COMMA lit[n8] => InstructionNode::Movhi { rd, n8 },

            IN  reg[rd] COMMA lit[n8] => InstructionNode::In  { rd, n8 },
            OUT lit[n8] COMMA reg[ra] => InstructionNode::Out { n8, ra },
        }

        raw_data: RawDataNode {
            bytes[b] => RawDataNode::Bytes(b),
            words[w] => RawDataNode::Words(w),
            SPACE LIT(bytes) => RawDataNode::Bytes(vec![LiteralNode::Constant(0);bytes as usize]),
            SPACE LIT(bytes) COMMA lit[data] => RawDataNode::Bytes(vec![data;bytes as usize]),
            EVEN => RawDataNode::WordAlign,
        }

        bytes: Vec<LiteralNode> {
            BYTE lit[b] => vec![b],
            bytes[mut bytes] COMMA lit[b] => {
                bytes.push(b);
                bytes
            }
        }

        words: Vec<LiteralNode> {
            WORD lit[w] => vec![w],
            words[mut words] COMMA lit[w] => {
                words.push(w);
                words
            }
        }

        reg: RegistryNode {
            REG(reg) => RegistryNode { reg }
        }

        lit: LiteralNode {
            LIT(val) => LiteralNode::Constant(val),
            symbol_ref[sr] => LiteralNode::SymbolRef(sr),
            function[f] => LiteralNode::Function(Box::new(f)),
        }

        label: LabelNode {
            IDENT(label) COLON => LabelNode { label }
        }

        symbol_ref: SymbolRefNode {
            IDENT(name) => SymbolRefNode { name }
        }

        function: FunctionNode {
            LO LPAR lit[l] RPAR => FunctionNode::Lo(l),
            HI LPAR lit[l] RPAR => FunctionNode::Hi(l),
        }

        constant: ConstantNode {
            SET IDENT(name) COMMA lit[value] => ConstantNode::new(name, value),
            IDENT(name) EQUALS lit[value] => ConstantNode::new(name, value),
        }
    }

    pub fn new(tokens: T) -> Self {
        Self { tokens }
    }

    pub fn parse(self) -> Result<ProgramNode, AssemblerMessage> {
        Self::_parse(self.tokens).map_err(|(element, description)| {
            if let Some((_, span)) = element {
                AssemblerMessage {
                    msg_type: AssemblerMessageType::ERROR,
                    description: description.into(),
                    span: span.into(),
                }
            } else {
                AssemblerMessage {
                    msg_type: AssemblerMessageType::ERROR,
                    description: description.into(),
                    span: None,
                }
            }
        })
    }
}
