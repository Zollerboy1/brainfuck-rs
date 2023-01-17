use clap::Parser as ArgumentParser;
use tempfile::Builder as TempFileBuilder;

use path_absolutize::*;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    process::Command,
};

use inkwell::{
    context::Context,
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};

use crate::{optimizer::Optimizer, parser::Parser, tok::Tokenizer};

mod code_gen;
mod instruction;
mod optimizer;
mod parser;
mod tok;

#[derive(ArgumentParser)]
#[command(author, version, about)]
/// A Brainfuck to executable compiler
struct Arguments {
    input_file: String,
    #[arg(short, long)]
    output_file: Option<String>,
    #[arg(short = 'O', long = "optimize")]
    optimize: bool,
}

impl Arguments {
    fn get_input_file(&self) -> PathBuf {
        Path::new(&self.input_file)
            .absolutize()
            .unwrap()
            .into_owned()
    }

    fn get_output_file(&self) -> PathBuf {
        match &self.output_file {
            Some(file) => Path::new(&file).absolutize().unwrap().into_owned(),
            None => self.get_input_file().with_extension(""),
        }
    }

    fn get_optimization_level(&self) -> OptimizationLevel {
        if self.optimize {
            OptimizationLevel::Default
        } else {
            OptimizationLevel::None
        }
    }

    fn get_optimization_passes(&self) -> &str {
        if self.optimize {
            "default<O2>"
        } else {
            "default<O0>"
        }
    }
}

impl Debug for Arguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arguments")
            .field("input_file", &self.get_input_file())
            .field("output_file", &self.get_output_file())
            .field("optimize", &self.optimize)
            .finish()
    }
}

fn main() {
    let args = Arguments::parse();
    let input_file_path = args.get_input_file();

    let input = std::fs::read_to_string(&input_file_path).unwrap();

    let tokenizer = Tokenizer::new(&input);
    let parser = Parser::new(tokenizer);

    let instructions = if args.optimize {
        Optimizer::new(parser).collect::<Vec<_>>()
    } else {
        parser.collect::<Vec<_>>()
    };

    let context = Context::create();
    let code_gen = code_gen::CodeGen::new(instructions, &input_file_path, &context);
    let module = code_gen.generate_module();

    Target::initialize_native(&InitializationConfig::default())
        .expect("Failed to initialize native target");

    let triple = TargetMachine::get_default_triple();
    let cpu = TargetMachine::get_host_cpu_name().to_string();
    let features = TargetMachine::get_host_cpu_features().to_string();

    let target = Target::from_triple(&triple).unwrap();
    let target_machine = target
        .create_target_machine(
            &triple,
            &cpu,
            &features,
            args.get_optimization_level(),
            RelocMode::PIC,
            CodeModel::Default,
        )
        .unwrap();

    module
        .run_passes(
            args.get_optimization_passes(),
            &target_machine,
            PassBuilderOptions::create(),
        )
        .unwrap();

    let object_file_path = TempFileBuilder::new()
        .prefix(&input_file_path.file_stem().unwrap())
        .suffix(".o")
        .tempfile()
        .unwrap()
        .into_temp_path();

    target_machine
        .write_to_file(module, FileType::Object, &object_file_path)
        .unwrap();

    let output_file = args.get_output_file();

    let helpers_file_path = Path::new("stdlib/helpers.c")
        .absolutize()
        .unwrap()
        .into_owned();

    let clang_status = Command::new("clang")
        .arg("-O2")
        .arg("-o")
        .arg(&output_file)
        .arg(&object_file_path)
        .arg(helpers_file_path)
        .status()
        .unwrap();

    assert!(clang_status.success());

    println!("Generated {}", output_file.to_str().unwrap());
}
