use crate::Span;
use easy_nodes::{node_system, Node};

node_system! {
    pub trait NodeVisitor<Span> {
        fn visit_program<Program>();
        fn visit_data_section<DataSection>();
        fn visit_text_section<TextSection>();
        fn visit_statement<Statement>();
        fn visit_instruction<Instruction>();
        fn visit_raw_data<RawData>();
        fn visit_registry<Registry>();
        fn visit_literal<Literal>();
        fn visit_label<Label>();
        fn visit_symbol_ref<SymbolRef>();
        fn visit_function<Function>();
        fn visit_constant<Constant>();
    }

    #[consumer = visit_program()]
    pub struct Program {
        pub data_section: Option<Node<Span, DataSection>>,
        pub text_section: Option<Node<Span, TextSection>>,
        pub constants: Vec<Node<Span, Constant>>,
    }

    #[consumer = visit_data_section()]
    pub struct DataSection {
        pub statements: Vec<Node<Span, Statement>>
    }

    #[consumer = visit_text_section()]
    pub struct TextSection {
        pub statements: Vec<Node<Span, Statement>>
    }

    #[consumer = visit_statement()]
    pub enum Statement {
        Instruction(Node<Span, Instruction>),
        Label(Node<Span, Label>),
        RawData(Node<Span, RawData>),
        Constant(Node<Span, Constant>),
    }

    #[consumer = visit_instruction()]
    pub enum Instruction {
        And { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Or { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Xor { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Not { rd: Node<Span, Registry>, ra: Node<Span, Registry> },
        Add { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Sub { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Sha { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Shl { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Cmplt { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Cmple { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Cmpeq { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Cmpltu { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Cmpleu { rd: Node<Span, Registry>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Addi { rd: Node<Span, Registry>, ra: Node<Span, Registry>, n6: Node<Span, Literal> },
        Ld { rd: Node<Span, Registry>, n6: Node<Span, Literal>, ra: Node<Span, Registry>},
        St { n6: Node<Span, Literal>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Ldb { rd: Node<Span, Registry>, n6: Node<Span, Literal>, ra: Node<Span, Registry>},
        Stb { n6: Node<Span, Literal>, ra: Node<Span, Registry>, rb: Node<Span, Registry> },
        Jalr { rd: Node<Span, Registry>, ra: Node<Span, Registry> },
        Bz { ra: Node<Span, Registry>, n8: Node<Span, Literal> },
        Bnz { ra: Node<Span, Registry>, n8: Node<Span, Literal> },
        Movi { rd: Node<Span, Registry>, n8: Node<Span, Literal> },
        Movhi { rd: Node<Span, Registry>, n8: Node<Span, Literal> },
        In { rd: Node<Span, Registry>, n8: Node<Span, Literal> },
        Out { n8: Node<Span, Literal>, ra: Node<Span, Registry> },
        Nop
    }

    #[consumer = visit_raw_data()]
    pub enum RawData {
        WordAlign,
        Bytes(Vec<Node<Span, Literal>>),
        Words(Vec<Node<Span, Literal>>),
    }

    #[consumer = visit_registry()]
    pub struct Registry {
        pub reg: u8
    }

    #[consumer = visit_literal()]
    pub enum Literal {
        Constant(u16),
        SymbolRef(Node<Span, SymbolRef>),
        Function(Node<Span, Function>),
    }

    #[consumer = visit_label()]
    pub struct Label {
        pub label: String
    }

    #[consumer = visit_symbol_ref()]
    pub struct SymbolRef {
        pub name: String
    }

    #[consumer = visit_function()]
    pub enum Function {
        Lo(Node<Span, Literal>),
        Hi(Node<Span, Literal>),
    }

    #[consumer = visit_constant()]
    pub struct Constant {
        pub name: String,
        pub value: Node<Span, Literal>
    }
}

impl Program {
    pub fn empty() -> Self {
        Self {
            text_section: None,
            data_section: None,
            constants: Vec::new(),
        }
    }
}

impl DataSection {
    pub fn empty() -> Self {
        Self { statements: vec![] }
    }
}

impl TextSection {
    pub fn empty() -> Self {
        Self { statements: vec![] }
    }
}

impl RawData {
    pub fn get_size(&self, pos: u16) -> u16 {
        match self {
            RawData::WordAlign => (pos % 2 != 0) as u16, 
            RawData::Bytes(data) => data.len() as u16,
            RawData::Words(data) => data.len() as u16 * 2,
        }
    }
}

impl Constant {
    pub fn new(name: String, value: Node<Span, Literal>) -> Self {
        Self { name, value }
    }
}
