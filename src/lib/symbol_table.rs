use std::cmp::max;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct SymbolTable {
    symbols: HashMap<String, SymbolTableEntry>,
    text_section: Section,
    data_section: Section,
}

struct Section {
    base_address: u16,
    length: u16,
}

pub struct SymbolTableEntry {
    value: u16,
    is_address: bool,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            data_section: Section {
                base_address: 0,
                length: 0,
            },
            text_section: Section {
                base_address: 0,
                length: 0,
            },
        }
    }

    pub fn put_constant(&mut self, name: String, value: u16) -> Result<(), String> {
        match self.symbols.entry(name) {
            Entry::Vacant(e) => {
                e.insert(SymbolTableEntry::new_constant(value));
                Ok(())
            },
            Entry::Occupied(e) => Err(format!("Symbol {} is already defined", e.key()))
        }
    }

    pub fn put_address(&mut self, name: String, value: u16) -> Result<(), String> {
        match self.symbols.entry(name) {
            Entry::Vacant(e) => {
                e.insert(SymbolTableEntry::new_address(value));
                Ok(())
            },
            Entry::Occupied(e) => Err(format!("Symbol {} is already defined", e.key()))
        }
    }

    pub fn get_symbol(&self, symbol: &str) -> Result<&SymbolTableEntry, String> {
        self.symbols
            .get(symbol)
            .ok_or(format!("Symbol {} isn't defined", symbol))
    }

    pub fn set_text_section(&mut self, base_address: u16, length: u16) {
        self.text_section = Section {
            base_address,
            length,
        };
    }

    pub fn set_data_section(&mut self, base_address: u16, length: u16) {
        self.data_section = Section {
            base_address,
            length,
        };
    }

    pub fn is_valid_layout(&self) -> bool {
        self.text_section.get_end_address() <= self.data_section.base_address
            || self.data_section.get_end_address() <= self.text_section.base_address
    }

    pub fn get_text_section_base_address(&self) -> u16 {
        self.text_section.base_address
    }

    pub fn get_data_section_base_address(&self) -> u16 {
        self.data_section.base_address
    }

    pub fn get_program_end_address(&self) -> u16 {
        max(
            self.data_section.get_end_address(),
            self.text_section.get_end_address(),
        )
    }
}

impl SymbolTableEntry {
    fn new_constant(value: u16) -> Self {
        Self {
            value,
            is_address: false,
        }
    }

    fn new_address(value: u16) -> Self {
        Self {
            value,
            is_address: true,
        }
    }

    pub fn is_address(&self) -> bool {
        self.is_address
    }

    pub fn get_value(&self) -> u16 {
        self.value
    }
}

impl Section {
    fn get_end_address(&self) -> u16 {
        self.base_address + self.length
    }
}
