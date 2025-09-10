use cargo_craft::{
    Craft,
    ClapExecuter,
    ExecutionResult::{Err, Ok},
};

fn main() {
    match Craft::main() {
        Ok(_) => {}
        Err(mut receipt, error) => {
            receipt.runtime_errors.push(error.clone());
            receipt.write_receipt().unwrap_or_default();
            eprintln!("{error}");
            std::process::exit(101);
        }
    }
}
