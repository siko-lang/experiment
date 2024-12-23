use core::panic;
use std::collections::{BTreeMap, BTreeSet};

use crate::siko::hir::BodyBuilder::BodyBuilder;
use crate::siko::hir::Data::Enum;
use crate::siko::hir::Function::{BlockId, BlockInfo, FieldInfo, InstructionKind, Variable, VariableName};
use crate::siko::location::Location::Location;
use crate::siko::location::Report::ReportContext;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::matchcompiler::Compiler::MatchCompiler;
use crate::siko::syntax::Expr::{BinaryOp, Expr, SimpleExpr, UnaryOp};
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use crate::siko::syntax::Statement::StatementKind;
use crate::siko::{hir::Function::Body, syntax::Statement::Block};

use super::Environment::Environment;
use super::Error::ResolverError;
use super::ModuleResolver::ModuleResolver;
use super::TypeResolver::TypeResolver;

fn createOpName(traitName: &str, method: &str) -> QualifiedName {
    let stdOps = Box::new(QualifiedName::Module("Std.Ops".to_string()));
    QualifiedName::Item(Box::new(QualifiedName::Item(stdOps.clone(), traitName.to_string())), method.to_string())
}

#[derive(Debug, Clone)]
struct LoopInfo {
    body: BlockId,
    exit: BlockId,
    var: Variable,
}

pub struct ExprResolver<'a> {
    pub ctx: &'a ReportContext,
    pub bodyBuilder: BodyBuilder,
    syntaxBlockId: u32,
    pub moduleResolver: &'a ModuleResolver<'a>,
    typeResolver: &'a TypeResolver<'a>,
    emptyVariants: &'a BTreeSet<QualifiedName>,
    pub variants: &'a BTreeMap<QualifiedName, QualifiedName>,
    pub enums: &'a BTreeMap<QualifiedName, Enum>,
    loopInfos: Vec<LoopInfo>,
    varIndices: BTreeMap<String, u32>,
}

impl<'a> ExprResolver<'a> {
    pub fn new(
        ctx: &'a ReportContext,
        moduleResolver: &'a ModuleResolver,
        typeResolver: &'a TypeResolver<'a>,
        emptyVariants: &'a BTreeSet<QualifiedName>,
        variants: &'a BTreeMap<QualifiedName, QualifiedName>,
        enums: &'a BTreeMap<QualifiedName, Enum>,
    ) -> ExprResolver<'a> {
        ExprResolver {
            ctx: ctx,
            bodyBuilder: BodyBuilder::new(),
            syntaxBlockId: 0,
            moduleResolver: moduleResolver,
            typeResolver: typeResolver,
            emptyVariants: emptyVariants,
            variants: variants,
            enums: enums,
            loopInfos: Vec::new(),
            varIndices: BTreeMap::new(),
        }
    }

    pub fn createSyntaxBlockId(&mut self) -> String {
        let blockId = format!("block{}", self.syntaxBlockId);
        self.syntaxBlockId += 1;
        blockId
    }

    pub fn indexVar(&mut self, mut var: Variable) -> Variable {
        let index = self.varIndices.entry(var.value.to_string()).or_insert(1);
        var.index = *index;
        *index += 1;
        var
    }

    fn processFieldAssign<'e>(&mut self, receiver: &Expr, name: &Identifier, env: &'e Environment<'e>, rhsId: Variable, location: Location) {
        let mut receiver = receiver;
        let mut fields: Vec<FieldInfo> = Vec::new();
        fields.push(FieldInfo {
            name: name.toString(),
            location: name.location.clone(),
            ty: None,
        });
        loop {
            match &receiver.expr {
                SimpleExpr::Value(name) => {
                    let value = env.resolve(&name.toString());
                    match value {
                        Some(value) => {
                            fields.reverse();
                            self.addInstruction(InstructionKind::FieldAssign(value.clone(), rhsId, fields), location.clone());
                            return;
                        }
                        None => {
                            ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                        }
                    }
                }
                SimpleExpr::SelfValue => {
                    let value = Variable {
                        value: VariableName::Arg(format!("self")),
                        location: receiver.location.clone(),
                        ty: None,
                        index: 0,
                    };
                    fields.reverse();
                    self.addInstruction(InstructionKind::FieldAssign(value.clone(), rhsId, fields), location.clone());
                    return;
                }
                SimpleExpr::FieldAccess(r, name) => {
                    receiver = r;
                    fields.push(FieldInfo {
                        name: name.toString(),
                        location: name.location.clone(),
                        ty: None,
                    });
                }
                _ => {
                    ResolverError::InvalidAssignment(location.clone()).report(self.ctx);
                }
            }
        }
    }

    fn resolveBlock<'e>(&mut self, block: &Block, env: &'e Environment<'e>, resultValue: Variable) {
        let syntaxBlock = self.createSyntaxBlockId();
        //println!("Resolving block {} with var {} current {}", syntaxBlock, resultValue, self.targetBlockId);
        let blockInfo = BlockInfo { id: syntaxBlock };
        self.bodyBuilder
            .current()
            .implicit()
            .addInstruction(InstructionKind::BlockStart(blockInfo.clone()), block.location.clone());
        let mut env = Environment::child(env);
        let mut lastHasSemicolon = false;
        let mut blockValue = self.createValue("block", block.location.clone());
        for (index, statement) in block.statements.iter().enumerate() {
            if index == block.statements.len() - 1 && statement.hasSemicolon {
                lastHasSemicolon = true;
            }
            match &statement.kind {
                StatementKind::Let(pat, rhs, ty) => {
                    let rhs = self.resolveExpr(rhs, &mut env);
                    if let Some(ty) = ty {
                        let ty = self.typeResolver.resolveType(ty);
                        self.bodyBuilder.setTypeInBody(rhs.clone(), ty);
                    }
                    self.resolvePattern(pat, &mut env, rhs);
                }
                StatementKind::Assign(lhs, rhs) => {
                    let rhsId = self.resolveExpr(rhs, &mut env);
                    match &lhs.expr {
                        SimpleExpr::Value(name) => {
                            let value = env.resolve(&name.toString());
                            match value {
                                Some(value) => {
                                    self.bodyBuilder.current().addAssign(value.clone(), rhsId, lhs.location.clone());
                                }
                                None => {
                                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                                }
                            }
                        }
                        SimpleExpr::FieldAccess(receiver, name) => {
                            self.processFieldAssign(receiver, name, &mut env, rhsId, lhs.location.clone());
                        }
                        _ => {
                            ResolverError::InvalidAssignment(lhs.location.clone()).report(self.ctx);
                        }
                    }
                }
                StatementKind::Expr(expr) => {
                    let var = self.resolveExpr(expr, &mut env);
                    blockValue = var;
                }
            }
        }
        if block.statements.is_empty() || lastHasSemicolon {
            blockValue = self.bodyBuilder.current().implicit().addUnit(block.location.clone());
        }
        if !block.doesNotReturn() {
            let blockValue = self.indexVar(blockValue);
            self.bodyBuilder
                .current()
                .implicit()
                .addAssign(resultValue.clone(), blockValue, block.location.clone());
        }
        self.bodyBuilder
            .current()
            .implicit()
            .addInstruction(InstructionKind::BlockEnd(blockInfo.clone()), block.location.clone());
    }

    pub fn addInstruction(&mut self, instruction: InstructionKind, location: Location) {
        self.bodyBuilder.addInstruction(instruction, location);
    }

    pub fn resolveExpr(&mut self, expr: &Expr, env: &mut Environment) -> Variable {
        match &expr.expr {
            SimpleExpr::Value(name) => match env.resolve(&name.name) {
                Some(mut var) => {
                    var.location = expr.location.clone();
                    self.indexVar(var)
                }
                None => {
                    ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
                }
            },
            SimpleExpr::SelfValue => Variable {
                value: VariableName::Arg(format!("self")),
                location: expr.location.clone(),
                ty: None,
                index: 0,
            },
            SimpleExpr::Name(name) => {
                let irName = self.moduleResolver.resolverName(name);
                if self.emptyVariants.contains(&irName) {
                    return self.bodyBuilder.current().addFunctionCall(irName, Vec::new(), expr.location.clone());
                }
                ResolverError::UnknownValue(name.name.clone(), name.location.clone()).report(self.ctx);
            }
            SimpleExpr::FieldAccess(receiver, name) => {
                let receiver = self.resolveExpr(receiver, env);
                self.bodyBuilder.current().addFieldRef(receiver, name.toString(), expr.location.clone())
            }
            SimpleExpr::Call(callable, args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    let argId = self.indexVar(argId);
                    irArgs.push(argId)
                }
                match &callable.expr {
                    SimpleExpr::Name(name) => {
                        let irName = self.moduleResolver.resolverName(name);
                        if self.enums.get(&irName).is_some() {
                            ResolverError::NotAConstructor(name.name.clone(), name.location.clone()).report(self.ctx);
                        }
                        return self.bodyBuilder.current().addFunctionCall(irName, irArgs, expr.location.clone());
                    }
                    SimpleExpr::Value(name) => {
                        if let Some(name) = env.resolve(&name.name) {
                            let valueRef = self.createValue("valueRef", expr.location.clone());
                            self.addInstruction(InstructionKind::ValueRef(valueRef.clone(), name), expr.location.clone());
                            let value = self.createValue("call", expr.location.clone());
                            self.addInstruction(
                                InstructionKind::DynamicFunctionCall(value.clone(), valueRef, irArgs),
                                expr.location.clone(),
                            );
                            value
                        } else {
                            let irName = self.moduleResolver.resolverName(name);
                            self.bodyBuilder.current().addFunctionCall(irName, irArgs, expr.location.clone())
                        }
                    }
                    _ => {
                        let callableId = self.resolveExpr(&callable, env);
                        let value = self.createValue("call", expr.location.clone());
                        self.addInstruction(
                            InstructionKind::DynamicFunctionCall(value.clone(), callableId, irArgs),
                            expr.location.clone(),
                        );
                        value
                    }
                }
            }
            SimpleExpr::MethodCall(receiver, name, args) => {
                let receiver = self.resolveExpr(&receiver, env);
                let receiver = self.indexVar(receiver);
                let value = self.createValue("call", expr.location.clone());
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    let argId = self.indexVar(argId);
                    irArgs.push(argId)
                }
                self.addInstruction(
                    InstructionKind::MethodCall(value.clone(), receiver, name.toString(), irArgs),
                    expr.location.clone(),
                );
                value
            }
            SimpleExpr::TupleIndex(receiver, index) => {
                let receiver = self.resolveExpr(&receiver, env);
                let receiver = self.indexVar(receiver);
                self.bodyBuilder
                    .current()
                    .addTupleIndex(receiver, index.parse().unwrap(), expr.location.clone())
            }
            SimpleExpr::For(_, _, _) => todo!(),
            SimpleExpr::Loop(pattern, init, body) => {
                let initId = self.resolveExpr(&init, env);
                let name = self.createValue("loopVar", expr.location.clone());
                self.addInstruction(InstructionKind::Bind(name.clone(), initId, true), init.location.clone());
                let mut loopBodyBuilder = self.bodyBuilder.createBlock();
                let mut loopExitBuilder = self.bodyBuilder.createBlock();
                let finalValue = self.createValue("finalValueRef", expr.location.clone());
                self.bodyBuilder.current().addJump(loopBodyBuilder.getBlockId(), expr.location.clone());
                let mut loopEnv = Environment::child(env);
                loopBodyBuilder.current();
                self.resolvePattern(pattern, &mut loopEnv, name.clone());
                self.loopInfos.push(LoopInfo {
                    body: loopBodyBuilder.getBlockId(),
                    exit: loopExitBuilder.getBlockId(),
                    var: name.clone(),
                });
                match &body.expr {
                    SimpleExpr::Block(block) => self.resolveBlock(block, &loopEnv, name.clone()),
                    _ => panic!("for body is not a block!"),
                };
                self.bodyBuilder
                    .current()
                    .implicit()
                    .addJump(loopBodyBuilder.getBlockId(), expr.location.clone());
                self.loopInfos.pop();
                loopExitBuilder.current();
                loopExitBuilder
                    .implicit()
                    .addInstruction(InstructionKind::ValueRef(finalValue.clone(), name), expr.location.clone());
                finalValue
            }
            SimpleExpr::BinaryOp(op, lhs, rhs) => {
                let lhsId = self.resolveExpr(lhs, env);
                let lhsId = self.indexVar(lhsId);
                let rhsId = self.resolveExpr(rhs, env);
                let rhsId = self.indexVar(rhsId);
                let name = match op {
                    BinaryOp::And => createOpName("And", "and"),
                    BinaryOp::Or => createOpName("Or", "or"),
                    BinaryOp::Add => createOpName("Add", "add"),
                    BinaryOp::Sub => createOpName("Sub", "sub"),
                    BinaryOp::Mul => createOpName("Mul", "mul"),
                    BinaryOp::Div => createOpName("Div", "div"),
                    BinaryOp::Equal => createOpName("PartialEq", "eq"),
                    BinaryOp::NotEqual => createOpName("PartialEq", "ne"),
                    BinaryOp::LessThan => createOpName("PartialOrd", "lessThan"),
                    BinaryOp::GreaterThan => createOpName("PartialOrd", "greaterThan"),
                    BinaryOp::LessThanOrEqual => createOpName("PartialOrd", "lessOrEqual"),
                    BinaryOp::GreaterThanOrEqual => createOpName("PartialOrd", "greaterOrEqual"),
                };
                let id = Identifier {
                    name: format!("{}", name),
                    location: expr.location.clone(),
                };
                let name = self.moduleResolver.resolverName(&id);
                self.bodyBuilder
                    .current()
                    .addFunctionCall(name, vec![lhsId, rhsId], expr.location.clone())
            }
            SimpleExpr::UnaryOp(op, rhs) => {
                let rhsId = self.resolveExpr(rhs, env);
                let name = match op {
                    UnaryOp::Not => createOpName("Not", "not"),
                };
                let id = Identifier {
                    name: format!("{}", name),
                    location: expr.location.clone(),
                };
                let name = self.moduleResolver.resolverName(&id);
                self.bodyBuilder.current().addFunctionCall(name, vec![rhsId], expr.location.clone())
            }
            SimpleExpr::Match(body, branches) => {
                let bodyId = self.resolveExpr(body, env);
                let mut matchResolver = MatchCompiler::new(self, bodyId, expr.location.clone(), body.location.clone(), branches.clone(), env);
                matchResolver.compile()
            }
            SimpleExpr::Block(block) => {
                let blockValue = self.createValue("blockValue", expr.location.clone());
                if !block.doesNotReturn() {
                    self.bodyBuilder
                        .current()
                        .implicit()
                        .addInstruction(InstructionKind::DeclareVar(blockValue.clone()), expr.location.clone());
                }
                self.resolveBlock(block, env, blockValue.clone());
                self.indexVar(blockValue)
            }
            SimpleExpr::Tuple(args) => {
                let mut irArgs = Vec::new();
                for arg in args {
                    let argId = self.resolveExpr(arg, env);
                    irArgs.push(argId)
                }
                self.bodyBuilder.current().addTuple(irArgs, expr.location.clone())
            }
            SimpleExpr::StringLiteral(v) => self.bodyBuilder.current().addStringLiteral(v.clone(), expr.location.clone()),
            SimpleExpr::IntegerLiteral(v) => self.bodyBuilder.current().addIntegerLiteral(v.clone(), expr.location.clone()),
            SimpleExpr::CharLiteral(v) => self.bodyBuilder.current().addCharLiteral(v.clone(), expr.location.clone()),
            SimpleExpr::Return(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.bodyBuilder.current().addUnit(expr.location.clone()),
                };
                self.bodyBuilder.current().addReturn(argId, expr.location.clone())
            }
            SimpleExpr::Break(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.bodyBuilder.current().addUnit(expr.location.clone()),
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.bodyBuilder.current().addAssign(info.var, argId, expr.location.clone());
                self.bodyBuilder.current().addJump(info.exit, expr.location.clone())
            }
            SimpleExpr::Continue(arg) => {
                let argId = match arg {
                    Some(arg) => self.resolveExpr(arg, env),
                    None => self.bodyBuilder.current().addUnit(expr.location.clone()),
                };
                let info = match self.loopInfos.last() {
                    Some(info) => info.clone(),
                    None => ResolverError::BreakOutsideLoop(expr.location.clone()).report(self.ctx),
                };
                self.bodyBuilder.current().addAssign(info.var, argId, expr.location.clone());
                self.bodyBuilder.current().addJump(info.body, expr.location.clone())
            }
            SimpleExpr::Ref(arg) => {
                let arg = self.resolveExpr(arg, env);
                self.bodyBuilder.current().addRef(arg, expr.location.clone())
            }
        }
    }

    pub fn createValue(&mut self, name: &str, location: Location) -> Variable {
        self.bodyBuilder.createValue(name, location)
    }

    fn resolvePattern(&mut self, pat: &Pattern, env: &mut Environment, root: Variable) {
        match &pat.pattern {
            SimplePattern::Named(_name, _args) => todo!(),
            SimplePattern::Bind(name, mutable) => {
                let new = self.createValue(&name.name, pat.location.clone());
                self.addInstruction(InstructionKind::Bind(new.clone(), root, *mutable), pat.location.clone());
                env.addValue(name.toString(), new);
            }
            SimplePattern::Tuple(args) => {
                for (index, arg) in args.iter().enumerate() {
                    let tupleValue = self.bodyBuilder.current().addTupleIndex(root.clone(), index as i32, pat.location.clone());
                    self.resolvePattern(arg, env, tupleValue);
                }
            }
            SimplePattern::StringLiteral(_) => todo!(),
            SimplePattern::IntegerLiteral(_) => todo!(),
            SimplePattern::Wildcard => {}
        }
    }

    pub fn resolve<'e>(&mut self, body: &Block, env: &'e Environment<'e>) {
        let mut blockBuilder = self.bodyBuilder.createBlock();
        blockBuilder.current();
        let functionResult = self.createValue("functionResult", body.location.clone());
        self.bodyBuilder
            .current()
            .implicit()
            .addInstruction(InstructionKind::DeclareVar(functionResult.clone()), body.location.clone());
        self.resolveBlock(body, env, functionResult.clone());
        self.bodyBuilder.current().implicit().addReturn(functionResult, body.location.clone());
        self.bodyBuilder.sortBlocks();
    }

    pub fn body(self) -> Body {
        self.bodyBuilder.build()
    }
}
