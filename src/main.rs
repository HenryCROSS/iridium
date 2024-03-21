pub mod repl;
pub mod vm;
pub mod instruction;

fn main() {
    let mut repl = repl::REPL::new();
    repl.run();
}
