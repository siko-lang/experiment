#![allow(non_snake_case)]
#![allow(dead_code)]

mod siko;

use siko::{
    backend::{DeadCodeEliminator::eliminateDeadCode, RemoveTuples::removeTuples},
    hir::Program::Program,
    hir_lowering::Lowering::lowerProgram,
    llvm::Generator::Generator,
    location::{FileManager::FileManager, Report::ReportContext},
    monomorphizer::Monomorphizer::Monomorphizer,
    parser::Parser::*,
    resolver::Resolver::Resolver,
    typechecker::Typechecker::Typechecker,
};

use std::{collections::BTreeMap, env::args};

fn typecheck(ctx: &ReportContext, mut program: Program) -> Program {
    let mut result = BTreeMap::new();
    for (_, f) in &program.functions {
        let moduleName = f.name.module();
        let traitMethodSelector = &program.traitMethodSelectors.get(&moduleName).expect("Trait method selector not found");
        let mut typechecker = Typechecker::new(ctx, &program, &traitMethodSelector);
        let typedFn = typechecker.run(f);
        //typedFn.dump();
        result.insert(typedFn.name.clone(), typedFn);
    }
    program.functions = result;
    program
}

// fn borrowcheck(program: &Program) {
//     let builder = DataFlowProfileBuilder::new(program);
//     let program = builder.process();
//     println!("{}", program);
//     for (_, f) in &program.functions {
//         if f.body.is_some() {
//             let mut borrowchecker = BorrowChecker::BorrowChecker::new(f);
//             borrowchecker.check();
//         }
//     }
// }

fn monomorphize(ctx: &ReportContext, program: Program) -> Program {
    let monomorphizer = Monomorphizer::new(ctx, program);
    monomorphizer.run()
}

fn main() {
    let ctx = ReportContext {};
    let fileManager = FileManager::new();
    let mut resolver = Resolver::new(&ctx);
    let mut parseOutput = false;
    let mut outputFile = "llvm.ll".to_string();
    for arg in args().skip(1) {
        if arg == "-o" {
            parseOutput = true;
            continue;
        }
        if parseOutput {
            outputFile = arg.clone();
            parseOutput = false;
            continue;
        }
        let fileId = fileManager.add(arg.clone());
        let mut parser = Parser::new(&ctx, fileId, arg.to_string());
        parser.parse();
        let modules = parser.modules();
        for m in modules {
            resolver.addModule(m);
        }
    }
    resolver.process();
    let program = resolver.ir();
    let program = typecheck(&ctx, program);
    //println!("after tychk\n{}", program);
    let program = eliminateDeadCode(&ctx, program);
    let program = monomorphize(&ctx, program);
    //println!("after mono\n{}", program);
    let program = removeTuples(&program);
    //let data_lifetime_inferer = DataLifeTimeInference::new(program);
    //let program = data_lifetime_inferer.process();
    //println!("after backend\n {}", program);
    let mut mir_program = lowerProgram(&program);
    //println!("mir\n{}", mir_program);
    let llvm_program = mir_program.process();
    let mut generator = Generator::new(outputFile, llvm_program);
    generator.dump().expect("llvm generator failed");
    //println!("after data lifetime\n{}", program);
    //borrowcheck(&program);
    //dataflow(&functions);
}
