#[derive(Debug, PartialEq, Clone)]
pub enum SymbolType {
    Label,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Symbol {
    name: String,
    offset: u32,
    symbol_type: SymbolType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}
impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType, offset: u32) -> Symbol {
        Symbol {
            name,
            symbol_type,
            offset,
        }
    }
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { symbols: vec![] }
    }

    pub fn has_symbol(&self, name: &str) -> bool {
        self.symbols.iter().any(|symbol| symbol.name == name)
    }

    pub fn add_symbol(&mut self, s: Symbol) {
        self.symbols.push(s);
    }

    pub fn symbol_value(&self, s: &str) -> Option<u32> {
        for symbol in &self.symbols {
            if symbol.name == s {
                return Some(symbol.offset);
            }
        }
        None
    }

    pub fn set_symbol_offset(&mut self, s: &str, offset: u32) -> bool {
        for symbol in &mut self.symbols {
            if symbol.name == s {
                symbol.offset = offset;
                return true;
            }
        }
        false

    }
}
