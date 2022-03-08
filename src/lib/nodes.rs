use crate::visitors::Visitor;
use std::fmt::Debug;

pub trait Node: Debug + Clone {
    fn accept<T: Default, E, V: Visitor<T, E>>(&self, v: &mut V) -> Result<T, E>;
}

macro_rules! node {
    (#[consumer = $method:ident()] $vis:vis $decl:ident $ident:ident $($tt:tt)?) => {
        #[derive(Debug, Clone)]
        $vis $decl $ident $($tt)?

        impl Node for $ident {
            fn accept<T: Default, E, V: Visitor<T, E>>(&self, v: &mut V) -> Result<T, E> {
                v.$method(self)
            }
        }
    }
}

macro_rules! nodes {
    ($(#[consumer = $method:ident()] $vis:vis $decl:ident $ident:ident $($tt:tt)? $(;)?)*) => {
        $(node! {
            #[consumer = $method()] $vis $decl $ident $($tt)?
        })*
    };
}

nodes! {
    #[consumer = visit_program()]
    pub struct ProgramNode {
        pub data_section: Option<DataSectionNode>,
        pub text_section: Option<TextSectionNode>,
        pub constants: Vec<ConstantNode>,
    }

    #[consumer = visit_data_section()]
    pub struct DataSectionNode {
        pub statements: Vec<StatementNode>
    }

    #[consumer = visit_text_section()]
    pub struct TextSectionNode {
        pub statements: Vec<StatementNode>
    }

    #[consumer = visit_statement()]
    pub enum StatementNode {
        Instruction(InstructionNode),
        Label(LabelNode),
        RawData(RawDataNode),
        Constant(ConstantNode),
    }

    #[consumer = visit_instruction()]
    pub enum InstructionNode {
        And { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Or { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Xor { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Not { rd: RegistryNode, ra: RegistryNode },
        Add { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Sub { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Sha { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Shl { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Cmplt { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Cmple { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Cmpeq { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Cmpltu { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Cmpleu { rd: RegistryNode, ra: RegistryNode, rb: RegistryNode },
        Addi { rd: RegistryNode, ra: RegistryNode, n6: LiteralNode },
        Ld { rd: RegistryNode, n6: LiteralNode, ra: RegistryNode},
        St { n6: LiteralNode, ra: RegistryNode, rb: RegistryNode },
        Ldb { rd: RegistryNode, n6: LiteralNode, ra: RegistryNode},
        Stb { n6: LiteralNode, ra: RegistryNode, rb: RegistryNode },
        Jalr { rd: RegistryNode, ra: RegistryNode },
        Bz { ra: RegistryNode, n8: LiteralNode },
        Bnz { ra: RegistryNode, n8: LiteralNode },
        Movi { rd: RegistryNode, n8: LiteralNode },
        Movhi { rd: RegistryNode, n8: LiteralNode },
        In { rd: RegistryNode, n8: LiteralNode },
        Out { n8: LiteralNode, ra: RegistryNode }
    }

    #[consumer = visit_raw_data()]
    pub enum RawDataNode {
        WordAlign,
        Bytes(Vec<LiteralNode>),
        Words(Vec<LiteralNode>),
    }

    #[consumer = visit_registry()]
    pub struct RegistryNode {
        pub reg: u8
    }

    #[consumer = visit_literal()]
    pub enum LiteralNode {
        Constant(u16),
        SymbolRef(SymbolRefNode),
        Function(Box<FunctionNode>)
    }

    #[consumer = visit_label()]
    pub struct LabelNode {
        pub label: String
    }

    #[consumer = visit_symbol_ref()]
    pub struct SymbolRefNode {
        pub name: String
    }

    #[consumer = visit_function()]
    pub enum FunctionNode {
        Lo(LiteralNode),
        Hi(LiteralNode)
    }

    #[consumer = visit_constant()]
    pub struct ConstantNode {
        pub name: String,
        pub value: LiteralNode,
    }
}

impl ProgramNode {
    pub fn empty() -> Self {
        Self {
            text_section: None,
            data_section: None,
            constants: Vec::new(),
        }
    }
}

impl DataSectionNode {
    pub fn empty() -> Self {
        Self { statements: vec![] }
    }
}

impl TextSectionNode {
    pub fn empty() -> Self {
        Self { statements: vec![] }
    }
}

impl RawDataNode {
    pub fn get_size(&self, pos: u16) -> u16 {
        match self {
            RawDataNode::WordAlign => {
                if pos % 2 == 0 {
                    0
                } else {
                    1
                }
            }
            RawDataNode::Bytes(data) => data.len() as u16,
            RawDataNode::Words(data) => data.len() as u16 * 2,
        }
    }
}

impl ConstantNode {
    pub fn new(name: String, value: LiteralNode) -> Self {
        Self { name, value }
    }
}
