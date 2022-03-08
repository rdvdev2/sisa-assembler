use crate::nodes::*;

mod machine_code_generator;
mod symbol_table_builder;

pub use machine_code_generator::MachineCodeGenerator;
pub use symbol_table_builder::SymbolTableBuilder;

macro_rules! visit_node {
    ($method:ident($ty:ty)) => {
        #[allow(unused_variables)]
        fn $method(&mut self, node: &$ty) -> Result<T, E> {
            Ok(Default::default())
        }
    };
}

macro_rules! visit_nodes {
    ($($method:ident($ty:ty);)*) => {
        $(visit_node!($method($ty));)*
    }
}

pub trait Visitor<T: Default = (), E = String> {
    visit_nodes! {
        visit_program(ProgramNode);
        visit_data_section(DataSectionNode);
        visit_text_section(TextSectionNode);
        visit_statement(StatementNode);
        visit_instruction(InstructionNode);
        visit_raw_data(RawDataNode);
        visit_registry(RegistryNode);
        visit_literal(LiteralNode);
        visit_label(LabelNode);
        visit_symbol_ref(SymbolRefNode);
        visit_function(FunctionNode);
        visit_constant(ConstantNode);
    }
}
