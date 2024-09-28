use std::collections::BTreeMap;

use crate::siko::{
    ir::Function::{Function, InstructionKind},
    qualifiedname::QualifiedName,
    util::DependencyProcessor::{processDependencies, DependencyGroup},
};

pub struct DataGroup {}

pub fn createFunctionGroups(
    functions: &BTreeMap<QualifiedName, Function>,
) -> Vec<DependencyGroup<QualifiedName>> {
    let mut dependency_map = BTreeMap::new();

    for (name, f) in functions {
        let deps = dependency_map
            .entry(name.clone())
            .or_insert_with(|| Vec::new());
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    match &instruction.kind {
                        InstructionKind::FunctionCall(name, _) => {
                            deps.push(name.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let groups = processDependencies(&dependency_map);
    for group in &groups {
        println!("function group {:?}", group);
    }
    groups
}
