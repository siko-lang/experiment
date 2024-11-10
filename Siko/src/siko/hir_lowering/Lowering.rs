use crate::siko::{
    hir::{
        Data::{Class as HirClass, Enum as HirEnum},
        Function::{Block, BlockId, Function as HirFunction, FunctionKind, InstructionId, InstructionKind as HirInstructionKind},
        Program::Program as HirProgram,
        Type::Type as HirType,
    },
    mir::{
        Data::{Field as MirField, Struct, Union, Variant as MirVariant},
        Function::{
            Block as MirBlock, EnumCase as MirEnumCase, Function as MirFunction, FunctionKind as MirFunctionKind, Instruction,
            IntegerCase as MirIntegerCase, Param as MirParam, Value, Variable,
        },
        Program::Program as MirProgram,
        Type::Type as MirType,
    },
    qualifiedname::QualifiedName,
};

pub struct Builder<'a> {
    program: &'a HirProgram,
    function: &'a HirFunction,
}

impl<'a> Builder<'a> {
    pub fn new(program: &'a HirProgram, function: &'a HirFunction) -> Builder<'a> {
        Builder {
            program: program,
            function: function,
        }
    }

    fn buildInstructionVar(&self, id: &InstructionId) -> Variable {
        let i = self.function.getInstruction(*id);
        let ty = lowerType(i.ty.as_ref().expect("no ty"), &self.program);
        let name = format!("b{}i{}", id.getBlockById().id, id.getId() + 1);
        Variable { name: name, ty: ty }
    }

    fn getBlockName(&self, blockId: BlockId) -> String {
        format!("block{}", blockId.id)
    }

    fn lowerBlock(&mut self, hirBlock: &Block) -> Option<MirBlock> {
        if hirBlock.instructions.is_empty() {
            return None;
        }
        let mut block = MirBlock {
            id: self.getBlockName(hirBlock.id),
            instructions: Vec::new(),
        };
        // for instruction in &hirBlock.instructions {
        //     if let HirInstructionKind::Drop(_) = instruction.kind {
        //         continue;
        //     }
        //     let idVar = self.buildInstructionVar(&instruction.id);
        //     match &instruction.kind {
        //         HirInstructionKind::FunctionCall(name, args) => {
        //             let args = args.iter().map(|id| self.buildInstructionVar(id)).collect();
        //             block.instructions.push(Instruction::Declare(idVar.clone()));
        //             block.instructions.push(Instruction::Call(idVar, convertName(name), args));
        //         }
        //         HirInstructionKind::Tuple(_) => {
        //             unreachable!()
        //         }
        //         HirInstructionKind::Drop(_) => {}
        //         HirInstructionKind::DeclareVar(var) => {
        //             let i = self.function.getInstruction(instruction.id);
        //             let ty = lowerType(i.ty.as_ref().expect("no ty"), &self.program);
        //             let var = Variable { name: var.clone(), ty: ty };
        //             block.instructions.push(Instruction::Declare(var.clone()));
        //         }
        //         HirInstructionKind::ValueRef(name) => {
        //             let i = self.function.getInstruction(instruction.id);
        //             let ty = lowerType(i.ty.as_ref().expect("no ty"), &self.program);
        //             let var = Variable {
        //                 name: name.getValue(),
        //                 ty: ty,
        //             };
        //             block.instructions.push(Instruction::Declare(idVar.clone()));
        //             block.instructions.push(Instruction::Memcpy(var, idVar));
        //         }
        //         HirInstructionKind::Assign(name, rhs) => {
        //             let rhsI = self.function.getInstruction(*rhs);
        //             let ty = lowerType(rhsI.ty.as_ref().expect("no ty"), &self.program);
        //             let var = Variable {
        //                 name: name.getValue(),
        //                 ty: ty,
        //             };
        //             let rhs = self.buildInstructionVar(rhs);
        //             block.instructions.push(Instruction::Memcpy(rhs, var));
        //         }
        //         HirInstructionKind::Bind(var, rhs, _) => {
        //             let i = self.function.getInstruction(*rhs);
        //             let ty = lowerType(i.ty.as_ref().expect("no ty"), &self.program);
        //             let var = Variable {
        //                 name: var.to_string(),
        //                 ty: ty,
        //             };
        //             let rhs = self.buildInstructionVar(rhs);
        //             block.instructions.push(Instruction::Declare(var.clone()));
        //             block.instructions.push(Instruction::Memcpy(rhs, var));
        //         }
        //         HirInstructionKind::Jump(blockId) => {
        //             block.instructions.push(Instruction::Jump(self.getBlockName(*blockId)));
        //         }
        //         HirInstructionKind::Return(v) => {
        //             block.instructions.push(Instruction::Return(Value::Var(self.buildInstructionVar(v))));
        //         }
        //         HirInstructionKind::IntegerLiteral(v) => {
        //             block.instructions.push(Instruction::Declare(idVar.clone()));
        //             block.instructions.push(Instruction::IntegerLiteral(idVar, v.to_string()));
        //         }
        //         HirInstructionKind::StringLiteral(v) => {
        //             block.instructions.push(Instruction::Declare(idVar.clone()));
        //             block.instructions.push(Instruction::StringLiteral(idVar, v.to_string()));
        //         }
        //         HirInstructionKind::EnumSwitch(root, cases) => {
        //             let root = self.buildInstructionVar(root);
        //             let mut mirCases = Vec::new();
        //             for case in cases {
        //                 let mirCase = MirEnumCase {
        //                     index: case.index,
        //                     branch: self.getBlockName(case.branch),
        //                 };
        //                 mirCases.push(mirCase);
        //             }
        //             block.instructions.push(Instruction::EnumSwitch(root, mirCases));
        //         }
        //         HirInstructionKind::IntegerSwitch(root, cases) => {
        //             let root = self.buildInstructionVar(root);
        //             let mut mirCases = Vec::new();
        //             for case in cases {
        //                 let mirCase = MirIntegerCase {
        //                     value: case.value.clone(),
        //                     branch: self.getBlockName(case.branch),
        //                 };
        //                 mirCases.push(mirCase);
        //             }
        //             block.instructions.push(Instruction::IntegerSwitch(root, mirCases));
        //         }
        //         HirInstructionKind::Transform(root, index, _) => {
        //             let root = self.buildInstructionVar(root);
        //             block.instructions.push(Instruction::Transform(idVar, root, *index));
        //         }
        //         HirInstructionKind::TupleIndex(root, index) => {
        //             let root = self.buildInstructionVar(root);
        //             block.instructions.push(Instruction::GetFieldRef(idVar, root, *index));
        //         }
        //         HirInstructionKind::FieldRef(root, name) => {
        //             let i = self.function.getInstruction(*root);
        //             let className = i.ty.as_ref().expect("no type").getName().expect("no name for field ref root");
        //             let c = self.program.classes.get(&className).expect("class not found");
        //             let (_, index) = c.getField(name);
        //             let root = self.buildInstructionVar(root);
        //             block.instructions.push(Instruction::GetFieldRef(idVar, root, index));
        //         }
        //         HirInstructionKind::Ref(arg) => {
        //             let arg = self.buildInstructionVar(arg);
        //             block.instructions.push(Instruction::Declare(idVar.clone()));
        //             block.instructions.push(Instruction::Reference(idVar, arg));
        //         }
        //         HirInstructionKind::Noop => {}
        //         k => panic!("NYI {}", k),
        //     }
        // }
        Some(block)
    }

    fn lowerFunction(&mut self) -> MirFunction {
        let mut args = Vec::new();
        for arg in &self.function.params {
            let arg = MirParam {
                name: format!("{}", arg.getName()),
                ty: lowerType(&arg.getType(), &self.program),
            };
            args.push(arg);
        }

        let kind = match self.function.kind {
            FunctionKind::ClassCtor => MirFunctionKind::ClassCtor,
            FunctionKind::UserDefined => {
                let mut blocks = Vec::new();
                if let Some(body) = self.function.body.clone() {
                    for block in &body.blocks {
                        if let Some(mirBlock) = self.lowerBlock(block) {
                            blocks.push(mirBlock);
                        }
                    }
                }
                MirFunctionKind::UserDefined(blocks)
            }
            FunctionKind::VariantCtor(i) => MirFunctionKind::VariantCtor(i),
            FunctionKind::Extern => MirFunctionKind::Extern,
        };
        let mirFunction = MirFunction {
            name: convertName(&self.function.name),
            args: args,
            result: lowerType(&self.function.result, &self.program),
            kind: kind,
        };
        mirFunction
    }
}

pub fn convertName(name: &QualifiedName) -> String {
    format!(
        "{}",
        name.toString()
            .replace(".", "_")
            .replace("(", "")
            .replace(")", "")
            .replace(",", "_")
            .replace(" ", "_")
            .replace("#", "_")
    )
}

pub fn lowerType(ty: &HirType, program: &HirProgram) -> MirType {
    match ty {
        HirType::Named(name, _, _) => {
            if program.classes.get(name).is_some() {
                MirType::Struct(convertName(name))
            } else {
                MirType::Union(convertName(name))
            }
        }
        HirType::Tuple(_) => unreachable!("Tuple in MIR"),
        HirType::Function(_, _) => todo!(),
        HirType::Var(_) => unreachable!("Type variable in MIR"),
        HirType::Reference(ty, _) => MirType::Ptr(Box::new(lowerType(ty, program))),
        HirType::SelfType => todo!(),
        HirType::Never => MirType::Void,
    }
}

pub fn lowerClass(c: &HirClass, program: &HirProgram) -> Struct {
    //println!("Lowering class {}", c.name);
    let mut fields = Vec::new();
    if c.name.toString() == "Int.Int" {
        fields.push(MirField {
            name: "value".to_string(),
            ty: MirType::Int64,
        });
    }
    if c.name.toString() == "String.String" {
        fields.push(MirField {
            name: "value".to_string(),
            ty: MirType::Ptr(Box::new(MirType::Int8)),
        });
        fields.push(MirField {
            name: "length".to_string(),
            ty: MirType::Int64,
        });
    }

    for f in &c.fields {
        let mirField = MirField {
            name: f.name.clone(),
            ty: lowerType(&f.ty, program),
        };
        fields.push(mirField);
    }
    Struct {
        name: convertName(&c.name),
        fields: fields,
        size: 0,
        alignment: 0,
    }
}

pub fn lowerEnum(e: &HirEnum, program: &HirProgram) -> Union {
    let mut variants = Vec::new();

    for v in &e.variants {
        assert_eq!(v.items.len(), 1);
        let mirVariant = MirVariant {
            name: convertName(&v.name),
            ty: lowerType(&v.items[0], program),
        };
        variants.push(mirVariant);
    }
    Union {
        name: convertName(&e.name),
        variants: variants,
        size: 0,
        alignment: 0,
        payloadSize: 0,
    }
}

pub fn lowerProgram(program: &HirProgram) -> MirProgram {
    let mut mirProgram = MirProgram::new();

    //println!("Lowering classes");

    for (n, c) in &program.classes {
        let c = lowerClass(c, program);
        mirProgram.structs.insert(convertName(n), c);
    }

    //println!("Lowering enums");

    for (n, e) in &program.enums {
        let u = lowerEnum(e, program);
        mirProgram.unions.insert(convertName(n), u);
    }

    //println!("Lowering functions");

    for (_, function) in &program.functions {
        let mut builder = Builder::new(program, function);
        let f = builder.lowerFunction();
        mirProgram.functions.push(f);
    }

    mirProgram
}
