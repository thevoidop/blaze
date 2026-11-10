mod ast;
mod codegen;
mod interpreter;
mod lexer;
mod parser;
mod semantic;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage:\n  blaze <file.blz>        (Run via Interpreter)\n  blaze build <file.blz>  (Compile to native binary)"
        );
        std::process::exit(1);
    }

    let (is_build, filepath) = if args[1] == "build" {
        if args.len() < 3 {
            eprintln!("Usage: blaze build <file.blz>");
            std::process::exit(1);
        }
        (true, &args[2])
    } else {
        (false, &args[1])
    };

    let source_code = fs::read_to_string(filepath).expect("Failed to read file");

    let mut lexer = lexer::Lexer::new(&source_code);
    let tokens = lexer.tokenize();

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse();

    let mut analyzer = semantic::SemanticAnalyzer::new();
    analyzer.analyze(&ast);

    if is_build {
        let binary_name = filepath.replace(".blz", "");
        let mut codegen = codegen::NativeCodegen::new();
        codegen.generate(&ast, &binary_name);
    } else {
        let mut interp = interpreter::Interpreter::new();
        interp.execute(ast);
    }
}
