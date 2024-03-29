pub mod directive_parsers;
pub mod instruction_parsers;
pub mod label_parsers;
pub mod opcode;
pub mod opcode_parsers;
pub mod operand_parser;
pub mod program_parsers;
pub mod register_parser;
pub mod symbols;

use byteorder::{LittleEndian, WriteBytesExt};

use crate::instruction::Opcode;

use self::{
    instruction_parsers::AssemblerInstruction,
    program_parsers::{program, Program},
    symbols::{Symbol, SymbolTable, SymbolType},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
    IrString { name: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerPhase {
    First,
    Second,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerSection {
    Code,
    Data,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerError {
    NoSegmentDeclarationFound { instruction: u32 },
    StringConstantDeclaredWithoutLabel { instruction: u32 },
    SymbolAlreadyDeclared,
    UnknownDirectiveFound { directive: String },
    NonOpcodeInOpcodeField,
    InsufficientSections,
    ParseError { error: String },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assembler {
    pub phase: AssemblerPhase,
    pub symbols: SymbolTable,
    /// The read-only data section constants are put in
    pub ro: Vec<u8>,
    /// The compiled bytecode generated from the assembly instructions
    pub bytecode: Vec<u8>,
    /// Tracks the current offset of the read-only section
    ro_offset: u32,
    /// A list of all the sections we've seen in the code
    sections: Vec<AssemblerSection>,
    /// The current section the assembler is in
    current_section: Option<AssemblerSection>,
    /// The current instruction the assembler is converting to bytecode
    current_instruction: u32,
    /// Any errors we find along the way. At the end, we'll present them to the user.
    errors: Vec<AssemblerError>,
}

pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];
pub const PIE_HEADER_LENGTH: usize = 64;

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
            ro: vec![],
            bytecode: vec![],
            ro_offset: 0,
            sections: vec![],
            current_section: None,
            current_instruction: 0,
            errors: vec![],
        }
    }

    pub fn assemble(&mut self, raw: &str) -> Result<Vec<u8>, Vec<AssemblerError>> {
        match program(raw) {
            Ok((_remainder, program)) => {
                self.process_first_phase(&program);

                if !self.errors.is_empty() {
                    // TODO: Can we avoid a clone here?
                    println!(
                        "Errors were found in the first parsing phase: {:?}",
                        self.errors
                    );
                    return Err(self.errors.clone());
                };

                // First get the header so we can smush it into the bytecode letter
                let mut assembled_program = self.write_pie_header();

                // If we accumulated any errors in the first pass, return them and don't try to do the second pass
                if !self.errors.is_empty() {
                    // TODO: Can we avoid a clone here?
                    return Err(self.errors.clone());
                };

                // Make sure that we have at least one data section and one code section
                if self.sections.len() != 2 {
                    // TODO: Detail out which one(s) are missing
                    println!("Did not find at least two sections.");
                    self.errors.push(AssemblerError::InsufficientSections);
                    // TODO: Can we avoid a clone here?
                    return Err(self.errors.clone());
                }

                // append the strings
                assembled_program.append(&mut self.ro);

                let mut body = self.process_second_phase(&program);

                // Merge the header with the populated body vector
                assembled_program.append(&mut body);
                Ok(assembled_program)
            }
            Err(e) => {
                println!("There was an error parsing the code: {:?}", e);
                Err(vec![AssemblerError::ParseError {
                    error: e.to_string(),
                }])
            }
        }
    }

    /// Handles the declaration of a label such as:
    /// hello: .asciiz 'Hello'
    fn process_label_declaration(&mut self, i: &AssemblerInstruction) {
        // Check if the label is None or String
        let name = match i.get_label_name() {
            Some(name) => name,
            None => {
                self.errors
                    .push(AssemblerError::StringConstantDeclaredWithoutLabel {
                        instruction: self.current_instruction,
                    });
                return;
            }
        };

        // Check if label is already in use (has an entry in the symbol table)
        // TODO: Is there a cleaner way to do this?
        if self.symbols.has_symbol(&name) {
            self.errors.push(AssemblerError::SymbolAlreadyDeclared);
            return;
        }

        // If we make it here, it isn't a symbol we've seen before, so stick it in the table
        let symbol = Symbol::new(name, SymbolType::Label, (self.current_instruction * 4) + 60);
        self.symbols.add_symbol(symbol);
    }

    /// Handles a declaration of a section header, such as:
    /// .code
    fn process_section_header(&mut self, header_name: &str) {
        let new_section: AssemblerSection = header_name.into();
        // Only specific section names are allowed
        if new_section == AssemblerSection::Unknown {
            println!(
                "Found an section header that is unknown: {:#?}",
                header_name
            );
            return;
        }
        // TODO: Check if we really need to keep a list of all sections seen
        self.sections.push(new_section.clone());
        self.current_section = Some(new_section);
    }

    fn process_directive(&mut self, i: &AssemblerInstruction) {
        // First let’s make sure we have a parseable name
        let directive_name = match i.get_directive_name() {
            Some(name) => name,
            None => {
                println!("Directive has an invalid name: {:?}", i);
                return;
            }
        };
        // Now check if there were any operands.
        if i.has_operands() {
            // If it _does_ have operands, we need to figure out which directive it was
            match directive_name.as_ref() {
                // If this is the operand, we're declaring a null terminated string
                "asciiz" => {
                    self.handle_asciiz(i);
                }
                "integer" => {
                    self.handle_integer(i);
                }
                _ => {
                    self.errors.push(AssemblerError::UnknownDirectiveFound {
                        directive: directive_name.clone(),
                    });
                    return;
                }
            }
        } else {
            // If there were not any operands, (e.g., `.code`), then we know it is a section header
            self.process_section_header(&directive_name);
        }
    }

    fn process_first_phase(&mut self, p: &Program) {
        // Iterate over every instruction, even though in the first phase we care
        // about labels and directives but nothing else
        for i in &p.instructions {
            if i.is_label() {
                // TODO: Factor this out into another function? Put it in `process_label_declaration`?
                if self.current_section.is_some() {
                    // If we have hit a segment header already (e.g., `.code`) then we are ok
                    self.process_label_declaration(&i);
                } else {
                    // If we have *not* hit a segment header yet, then we have a label outside of a segment,
                    // which is not allowed
                    self.errors.push(AssemblerError::NoSegmentDeclarationFound {
                        instruction: self.current_instruction,
                    });
                }
            }

            if i.is_directive() {
                self.process_directive(i);
            }

            // This is used to keep track of which instruction we hit an error on
            // TODO: Do we really need to track this?
            self.current_instruction += 1;
        }
        // Once we're done with this function, set the phase to second
        self.phase = AssemblerPhase::Second;
    }

    /// Runs the second pass of the assembler
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        // Restart the counting of instructions
        self.current_instruction = 0;
        // We're going to put the bytecode meant to be executed in a separate Vec so we can do some post-processing and then merge it with the header and read-only sections
        // Examples could be optimizations, additional checks, whatever
        let mut program = vec![];
        // Same as in first pass, except in the second pass we care about opcodes and directives
        for i in &p.instructions {
            if i.is_opcode() {
                // Opcodes know how to properly transform themselves into 32-bits, so we can just call `to_bytes` and append to our program
                let mut bytes = i.to_bytes(&self.symbols);
                program.append(&mut bytes);
            }
            if i.is_directive() {
                // In this phase, we can have directives but of different types than we care about in the first pass. The Directive itself can check which pass the Assembler
                // is in and decide what to do about it
                self.process_directive(i);
            }
            self.current_instruction += 1
        }
        program
    }

    fn extract_labels(&mut self, p: &Program) {
        let mut c = 0;
        for i in &p.instructions {
            if i.is_label() {
                match i.get_label_name() {
                    Some(name) => {
                        let symbol = Symbol::new(name, SymbolType::Label, c);
                        self.symbols.add_symbol(symbol);
                    }
                    None => {}
                };
            }
            c += 4;
        }
    }

    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            header.push(byte.clone());
        }
        while header.len() < PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }

        let mut data = vec![];
        // number of bytes
        data.write_u32::<LittleEndian>(self.ro.len() as u32)
            .unwrap();

        // end address I guess? need to be fixed
        let end = PIE_HEADER_LENGTH + 8 + self.ro.len();
        data.write_u32::<LittleEndian>(end as u32)
            .unwrap();
        header.append(&mut data);

        header
    }

    /// Handles a declaration of a null-terminated string:
    /// hello: .asciiz 'Hello!'
    fn handle_asciiz(&mut self, i: &AssemblerInstruction) {
        // Being a constant declaration, this is only meaningful in the first pass
        if self.phase != AssemblerPhase::First {
            return;
        }

        // In this case, operand1 will have the entire string we need to read in to RO memory
        match i.get_string_constant() {
            Some(s) => {
                match i.get_label_name() {
                    Some(name) => {
                        self.symbols.set_symbol_offset(&name, self.ro_offset);
                    }
                    None => {
                        // This would be someone typing:
                        // .asciiz 'Hello'
                        println!("Found a string constant with no associated label!");
                        return;
                    }
                };
                // We'll read the string into the read-only section byte-by-byte
                for byte in s.as_bytes() {
                    self.ro.push(*byte);
                    self.ro_offset += 1;
                }

                // This is the null termination bit we are using to indicate a string has ended
                self.ro.push(0);
                self.ro_offset += 1;
            }
            None => {
                // This just means someone typed `.asciiz` for some reason
                println!("String constant following an .asciiz was empty");
            }
        }
    }

    fn handle_integer(&mut self, i: &AssemblerInstruction){
        // Being a constant declaration, this is only meaningful in the first pass
        if self.phase != AssemblerPhase::First {
            return;
        }

        // In this case, operand1 will have the entire string we need to read in to RO memory
        match i.get_i32_constant() {
            Some(s) => {
                match i.get_label_name() {
                    Some(name) => {
                        self.symbols.set_symbol_offset(&name, self.ro_offset);
                    }
                    None => {
                        // This would be someone typing:
                        // .asciiz 'Hello'
                        println!("Found a string constant with no associated label!");
                        return;
                    }
                };
                
                let mut wtr = vec![];
                // TODO: Remove unwrap?
                wtr.write_i32::<LittleEndian>(s).unwrap();
                for byte in &wtr {
                    self.ro.push(*byte);
                    self.ro_offset += 1;
                }
            }
            None => {
                // This just means someone typed `.asciiz` for some reason
                println!("integer constant following an .integer was empty");
            }
        }
    }
}

impl From<&str> for AssemblerSection {
    fn from(header_name: &str) -> Self {
        match header_name {
            "code" => AssemblerSection::Code,
            "data" => AssemblerSection::Data,
            _ => AssemblerSection::Unknown,
        }
    }
}

mod tests {
    use crate::vm::VM;

    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new("test".to_string(), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(true, v.is_some());
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }

    #[test]
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        .code
        load $0 #100
        load $1 #1
        load $2 #0
        test: inc $0
        neq $0 $2
        jeq @test
        hlt
        ";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 93); // somehow cannot get 96
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 93);

        println!("{:?}", vm.program);
        println!("{:?}", vm.program.len());
    }

    #[test]
    /// Simple test of data that goes into the read only section
    fn test_code_start_offset_written() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        hello: .asciiz 'Hello'
        .code
        load $0 #100
        load $1 #1
        load $2 #0
        test: inc $0
        neq $0 $2
        jeq @test
        hlt
        ";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
        let unwrapped = program.unwrap();
        assert_eq!(unwrapped[64], 6);
        println!("{:?}", unwrapped);
    }

    #[test]
    /// Simple test of data that goes into the read only section
    fn test_ro_data_asciiz() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        test: .asciiz 'This is a test'
        .code
        ";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
    }

    #[test]
    /// Simple test of data that goes into the read only section
    fn test_ro_data_i32() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        test: .integer #300
        .code
        ";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
    }

    #[test]
    /// This tests that a section name that isn't `code` or `data` throws an error
    fn test_bad_ro_data() {
        let mut asm = Assembler::new();
        let test_string = r"
        .code
        test: .asciiz 'This is a test'
        .wrong
        ";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), false);
    }

    #[test]
    /// Tests that code which does not declare a segment first does not work
    fn test_first_phase_no_segment() {
        let mut asm = Assembler::new();
        let test_string = "hello: .asciiz 'Fail'";
        let result = program(test_string);
        assert_eq!(result.is_ok(), true);
        let (_, mut p) = result.unwrap();
        asm.process_first_phase(&mut p);
        assert_eq!(asm.errors.len(), 1);
    }

    #[test]
    /// Tests that code inside a proper segment works
    fn test_first_phase_inside_segment() {
        let mut asm = Assembler::new();
        let test_string = r"
        .data
        test: .asciiz 'Hello'
        ";
        let result = program(test_string);
        assert_eq!(result.is_ok(), true);
        let (_, mut p) = result.unwrap();
        asm.process_first_phase(&mut p);
        assert_eq!(asm.errors.len(), 0);
    }
}
