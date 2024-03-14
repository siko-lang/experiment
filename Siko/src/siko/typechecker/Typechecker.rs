use std::{
    collections::{BTreeMap, BTreeSet},
    iter::zip,
};

use crate::siko::{
    ir::{
        Data::{Class, Enum},
        Function::{Function, InstructionId, InstructionKind, Parameter, ValueKind},
        Type::Type,
    },
    qualifiedname::QualifiedName,
    util::error,
};

use super::{Substitution::Substitution, TypeVarAllocator::TypeVarAllocator};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TypedId {
    Instruction(InstructionId),
    Value(String),
    SelfValue,
}

pub struct Typechecker<'a> {
    functions: &'a BTreeMap<QualifiedName, Function>,
    classes: &'a BTreeMap<QualifiedName, Class>,
    enums: &'a BTreeMap<QualifiedName, Enum>,
    allocator: TypeVarAllocator,
    substitution: Substitution,
    methodSources: BTreeMap<InstructionId, QualifiedName>,
    methodCalls: BTreeMap<InstructionId, InstructionId>,
    types: BTreeMap<TypedId, Type>,
}

impl<'a> Typechecker<'a> {
    pub fn new(
        functions: &'a BTreeMap<QualifiedName, Function>,
        classes: &'a BTreeMap<QualifiedName, Class>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> Typechecker<'a> {
        Typechecker {
            functions: functions,
            classes: classes,
            enums: enums,
            allocator: TypeVarAllocator::new(),
            substitution: Substitution::new(),
            methodSources: BTreeMap::new(),
            methodCalls: BTreeMap::new(),
            types: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, f: &Function) -> Function {
        self.initialize(f);
        self.check(f);
        self.verify(f);
        //self.dump(f);
        self.generate(f)
    }

    pub fn initialize(&mut self, f: &Function) {
        //println!("Initializing {}", f.name);
        for param in &f.params {
            match &param {
                Parameter::Named(name, ty, _) => {
                    self.types.insert(TypedId::Value(name.clone()), ty.clone());
                }
                Parameter::SelfParam(_, ty) => {
                    self.types.insert(TypedId::SelfValue, ty.clone());
                }
            }
        }
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    let ty = self.allocator.next();
                    self.types
                        .insert(TypedId::Instruction(instruction.id), ty.clone());
                    match &instruction.kind {
                        InstructionKind::Bind(name, _) => {
                            self.types
                                .insert(TypedId::Value(name.to_string()), self.allocator.next());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn getType(&self, id: &TypedId) -> Type {
        self.types.get(id).expect("No type found!").clone()
    }

    fn getInstructionType(&self, id: InstructionId) -> Type {
        self.types
            .get(&TypedId::Instruction(id))
            .expect("not type for instruction")
            .clone()
    }

    fn getValueType(&self, v: &String) -> Type {
        self.types
            .get(&TypedId::Value(v.clone()))
            .expect("not type for value")
            .clone()
    }

    fn unify(&mut self, ty1: Type, ty2: Type) {
        self.substitution.unify(&ty1, &ty2);
    }

    fn instantiateType(&mut self, ty: Type) -> Type {
        let vars = ty.collectVars(BTreeSet::new());
        let mut sub = Substitution::new();
        for var in &vars {
            sub.add(var.clone(), self.allocator.next());
        }
        sub.apply(&ty)
    }

    pub fn check(&mut self, f: &Function) {
        //println!("Typechecking {}", f.name);
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    //println!("Type checking {}", instruction);
                    match &instruction.kind {
                        InstructionKind::FunctionCall(name, args) => {
                            let f = self.functions.get(name).expect("Function not found");
                            let fnType = f.getType();
                            let fnType = self.instantiateType(fnType);
                            let (fnArgs, fnResult) = fnType.splitFnType();
                            if args.len() != fnArgs.len() {
                                error(format!("incorrect args"));
                            }
                            for (arg, fnArg) in zip(args, fnArgs) {
                                self.substitution
                                    .unify(&self.getInstructionType(*arg), &fnArg);
                            }
                            self.substitution
                                .unify(&self.getInstructionType(instruction.id), &fnResult);
                        }
                        InstructionKind::DynamicFunctionCall(callable, args) => {
                            match self.methodSources.get(callable) {
                                Some(name) => {
                                    let f = self.functions.get(name).expect("Function not found");
                                    let fnType = f.getType();
                                    let fnType = self.instantiateType(fnType);
                                    let (fnArgs, fnResult) = fnType.splitFnType();
                                    let mut newArgs = Vec::new();
                                    newArgs.push(*callable);
                                    newArgs.extend(args);
                                    if newArgs.len() != fnArgs.len() {
                                        error(format!("incorrect args"));
                                    }
                                    for (arg, fnArg) in zip(newArgs, fnArgs) {
                                        self.substitution
                                            .unify(&self.getInstructionType(arg), &fnArg);
                                    }
                                    self.substitution
                                        .unify(&self.getInstructionType(instruction.id), &fnResult);
                                    self.methodCalls.insert(instruction.id, *callable);
                                }
                                None => {
                                    let fnType = self.getInstructionType(*callable);
                                    let (fnArgs, fnResult) = fnType.splitFnType();
                                    if args.len() != fnArgs.len() {
                                        error(format!("incorrect args"));
                                    }
                                    for (arg, fnArg) in zip(args, fnArgs) {
                                        self.substitution
                                            .unify(&self.getInstructionType(*arg), &fnArg);
                                    }
                                    self.substitution
                                        .unify(&self.getInstructionType(instruction.id), &fnResult);
                                }
                            }
                        }
                        InstructionKind::If(cond, t, f) => {
                            self.substitution
                                .unify(&self.getInstructionType(*cond), &Type::getBoolType());
                            match f {
                                Some(f) => {
                                    self.substitution.unify(
                                        &self.getInstructionType(*t),
                                        &self.getInstructionType(*f),
                                    );
                                }
                                None => {
                                    self.substitution
                                        .unify(&self.getInstructionType(*t), &Type::getUnitType());
                                }
                            }
                            self.substitution.unify(
                                &self.getInstructionType(*t),
                                &self.getInstructionType(instruction.id),
                            );
                        }
                        InstructionKind::BlockRef(id) => {
                            let block = &body.blocks[id.id as usize];
                            let last = block
                                .instructions
                                .last()
                                .expect("Empty block in type check!");
                            self.substitution.unify(
                                &self.getInstructionType(last.id),
                                &self.getInstructionType(instruction.id),
                            );
                        }
                        InstructionKind::ValueRef(value, fields) => {
                            let mut receiverType = match &value {
                                ValueKind::Arg(name) => self.getValueType(name),
                                ValueKind::Implicit(id) => self.getInstructionType(*id),
                                ValueKind::Value(name, _) => self.getValueType(name),
                            };
                            if fields.is_empty() {
                                self.unify(receiverType, self.getInstructionType(instruction.id));
                            } else {
                                for (index, field) in fields.iter().enumerate() {
                                    let receiver = self.substitution.apply(&receiverType);
                                    match &receiver {
                                        Type::Named(name, _) => {
                                            // TODO
                                            if let Some(c) = self.classes.get(name) {
                                                let mut found = false;
                                                for f in &c.fields {
                                                    if f.name == *field {
                                                        found = true;
                                                        receiverType = f.ty.clone();
                                                        break;
                                                    }
                                                }
                                                if !found && index == fields.len() - 1 {
                                                    for m in &c.methods {
                                                        if m.name == *field {
                                                            found = true;
                                                            self.unify(
                                                                receiverType.clone(),
                                                                self.getInstructionType(
                                                                    instruction.id,
                                                                ),
                                                            );
                                                            self.methodSources.insert(
                                                                instruction.id,
                                                                m.fullName.clone(),
                                                            );
                                                            break;
                                                        }
                                                    }
                                                }
                                                if !found {
                                                    error(format!("field '{}' not found", field))
                                                }
                                            } else {
                                                error(format!("field receiver is not a class!"))
                                            }
                                        }
                                        Type::Var(_) => error(format!("Type annotation needed!")),
                                        ty => {
                                            error(format!("field receiver is not a class! {}", ty))
                                        }
                                    }
                                }
                            }
                        }
                        InstructionKind::Bind(name, rhs) => {
                            self.unify(self.getValueType(name), self.getInstructionType(*rhs));
                            self.substitution.unify(
                                &self.getInstructionType(instruction.id),
                                &Type::getUnitType(),
                            );
                        }
                        InstructionKind::Tuple(_) => todo!(),
                        InstructionKind::TupleIndex(_, _) => todo!(),
                        InstructionKind::StringLiteral(_) => todo!(),
                        InstructionKind::IntegerLiteral(_) => todo!(),
                        InstructionKind::CharLiteral(_) => todo!(),
                    }
                }
            }
        }
    }

    pub fn verify(&self, f: &Function) {
        if let Some(body) = &f.body {
            let fnType = f.getType();
            let publicVars = fnType.collectVars(BTreeSet::new());
            let mut vars = BTreeSet::new();
            for block in &body.blocks {
                for instruction in &block.instructions {
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = self.substitution.apply(&ty);
                    vars = ty.collectVars(vars);
                }
            }
            if vars != publicVars {
                self.dump(f);
                error(format!("type check/inference failed for {}", f.name));
            }
        }
    }

    pub fn dump(&self, f: &Function) {
        println!("Dumping {}", f.name);
        if let Some(body) = &f.body {
            for block in &body.blocks {
                for instruction in &block.instructions {
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = self.substitution.apply(&ty);
                    println!("{} : {}", instruction, ty);
                }
            }
        }
    }

    pub fn generate(&self, f: &Function) -> Function {
        println!("Generating {}", f.name);
        let mut result = f.clone();
        if let Some(body) = &mut result.body {
            for block in &mut body.blocks {
                for instruction in &mut block.instructions {
                    if self.methodSources.contains_key(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::ValueRef(v, fields) => {
                                let mut fields = fields.clone();
                                fields.pop();
                                instruction.kind = InstructionKind::ValueRef(v.clone(), fields);
                            }
                            kind => panic!(
                                "Unexpected instruction kind for method source while rewriting! {}",
                                kind.dump()
                            ),
                        }
                    }
                    if let Some(source) = self.methodCalls.get(&instruction.id) {
                        match &instruction.kind {
                            InstructionKind::DynamicFunctionCall(_, args) => {
                                let name = self
                                    .methodSources
                                    .get(source)
                                    .expect("Method not found for call!");
                                let mut newArgs = Vec::new();
                                newArgs.push(*source);
                                newArgs.extend(args);
                                instruction.kind =
                                    InstructionKind::FunctionCall(name.clone(), newArgs);
                            }
                            kind => panic!(
                                "Unexpected instruction kind for method call while rewriting! {}",
                                kind.dump()
                            ),
                        }
                    }
                    let ty = self.getType(&TypedId::Instruction(instruction.id));
                    let ty = self.substitution.apply(&ty);
                    println!("{} : {}", instruction, ty);
                }
            }
        }
        result
    }
}
