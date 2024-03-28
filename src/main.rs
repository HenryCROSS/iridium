use std::{env, fs::File, io::Read, path::Path};

#[macro_use]
extern crate nom;

pub mod assembler;
pub mod instruction;
pub mod repl;
pub mod vm;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let filename = args[1].as_str();
        let program = read_file(filename);
        let mut asm = assembler::Assembler::new();
        let mut vm = vm::VM::new();
        let program = asm.assemble(&program);
        match program {
            Some(p) => {
                vm.add_bytes(p);
                vm.run();
                std::process::exit(0);
            }
            None => {}
        }
    }else {
        start_repl();
    }
}

fn start_repl() {
    let mut repl = repl::REPL::new();
    repl.run();
}

fn read_file(tmp: &str) -> String {
    let filename = Path::new(tmp);
    match File::open(Path::new(&filename)) {
        Ok(mut fh) => {
            let mut contents = String::new();
            match fh.read_to_string(&mut contents) {
                Ok(_) => {
                    return contents;
                }
                Err(e) => {
                    println!("There was an error reading file: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("File not found: {:?}", e);
            std::process::exit(1)
        }
    }
}
