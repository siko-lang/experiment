use crate::siko::hir::Function::{BlockId, InstructionId};
use crate::siko::location::Location::Location;
use crate::siko::qualifiedname::QualifiedName;
use crate::siko::resolver::Error::ResolverError;
use crate::siko::resolver::ExprResolver::ExprResolver;
use crate::siko::syntax::Identifier::Identifier;
use crate::siko::syntax::Pattern::{Pattern, SimplePattern};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::iter::repeat;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataPath {
    Root,
    Tuple(Box<DataPath>, i64),
    TupleIndex(Box<DataPath>, i64),
    Variant(Box<DataPath>, QualifiedName, QualifiedName),
    IntegerLiteral(Box<DataPath>, String),
    StringLiteral(Box<DataPath>, String),
    Class(Box<DataPath>, QualifiedName),
    Wildcard(Box<DataPath>),
}

impl DataPath {
    fn isChild(&self, parent: &DataPath) -> bool {
        let mut selfParent = self.getParent();
        loop {
            if &selfParent == parent {
                return true;
            }
            if selfParent == DataPath::Root {
                return false;
            }
            selfParent = selfParent.getParent();
        }
    }

    fn getParent(&self) -> DataPath {
        match self {
            DataPath::Root => DataPath::Root,
            DataPath::Tuple(p, _) => *p.clone(),
            DataPath::TupleIndex(p, _) => *p.clone(),
            DataPath::Variant(p, _, _) => *p.clone(),
            DataPath::IntegerLiteral(p, _) => *p.clone(),
            DataPath::StringLiteral(p, _) => *p.clone(),
            DataPath::Class(p, _) => *p.clone(),
            DataPath::Wildcard(p) => *p.clone(),
        }
    }
}

impl fmt::Display for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataPath::Root => write!(f, "Root"),
            DataPath::Tuple(path, len) => write!(f, "{}/tuple{}", path, len),
            DataPath::TupleIndex(path, index) => write!(f, "{}.t{}", path, index),
            DataPath::Variant(path, name, _) => write!(f, "{}.{}", path, name),
            DataPath::IntegerLiteral(path, literal) => write!(f, "{}[int:{}]", path, literal),
            DataPath::StringLiteral(path, literal) => write!(f, "{}[str:\"{}\"]", path, literal),
            DataPath::Class(path, name) => write!(f, "{}.{}", path, name),
            DataPath::Wildcard(path) => write!(f, "{}._", path),
        }
    }
}

#[derive(Debug)]
pub enum DataType {
    Class(QualifiedName),
    Enum(QualifiedName),
    Tuple(i64),
    Integer,
    String,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataType::Class(name) => write!(f, "Class({})", name),
            DataType::Enum(name) => write!(f, "Enum({})", name),
            DataType::Tuple(size) => write!(f, "Tuple({})", size),
            DataType::Integer => write!(f, "Integer"),
            DataType::String => write!(f, "String"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecisionPath {
    decisions: Vec<DataPath>,
}

impl DecisionPath {
    pub fn new() -> DecisionPath {
        DecisionPath { decisions: Vec::new() }
    }

    pub fn add(&self, path: DataPath) -> DecisionPath {
        let mut d = self.clone();
        d.decisions.push(path);
        d
    }
}

impl fmt::Display for DecisionPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decisions = self
            .decisions
            .iter()
            .map(|path| format!("{}", path))
            .collect::<Vec<String>>()
            .join(" -> ");

        write!(f, "{}", decisions)
    }
}

pub struct MatchCompiler<'a, 'b> {
    resolver: &'a mut ExprResolver<'b>,
    bodyId: InstructionId,
    bodyLocation: Location,
    branches: Vec<Pattern>,
    errors: Vec<ResolverError>,
    nextVar: i32,
    nodes: BTreeMap<DecisionPath, Node>,
}

impl<'a, 'b> MatchCompiler<'a, 'b> {
    pub fn new(resolver: &'a mut ExprResolver<'b>, bodyId: InstructionId, bodyLocation: Location, branches: Vec<Pattern>) -> MatchCompiler<'a, 'b> {
        MatchCompiler {
            bodyLocation: bodyLocation,
            bodyId: bodyId,
            branches: branches,
            resolver: resolver,
            errors: Vec::new(),
            nextVar: 1,
            nodes: BTreeMap::new(),
        }
    }

    fn resolve(&self, pattern: &Pattern) -> Pattern {
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolverName(&origId);
                let id = Identifier {
                    name: name.toString(),
                    location: Location::empty(),
                };
                let args = args.iter().map(|p| self.resolve(p)).collect();
                Pattern {
                    pattern: SimplePattern::Named(id, args),
                    location: Location::empty(),
                }
            }
            SimplePattern::Bind(_, _) => pattern.clone(),
            SimplePattern::Tuple(args) => {
                let args = args.iter().map(|p| self.resolve(p)).collect();
                Pattern {
                    pattern: SimplePattern::Tuple(args),
                    location: Location::empty(),
                }
            }
            SimplePattern::StringLiteral(_) => pattern.clone(),
            SimplePattern::IntegerLiteral(_) => pattern.clone(),
            SimplePattern::Wildcard => pattern.clone(),
        }
    }

    fn generateChoices(&self, pattern: &Pattern) -> Vec<Pattern> {
        let wildcardPattern = Pattern {
            pattern: SimplePattern::Wildcard,
            location: Location::empty(),
        };
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolverName(&origId);
                let mut result = Vec::new();
                if let Some(enumName) = self.resolver.variants.get(&name) {
                    let e = self.resolver.enums.get(enumName).expect("enum not found");
                    for variant in &e.variants {
                        if variant.name == name {
                            continue;
                        }
                        let id = Identifier {
                            name: variant.name.toString(),
                            location: Location::empty(),
                        };

                        let args = repeat(wildcardPattern.clone()).take(variant.items.len()).collect();
                        let pat = Pattern {
                            pattern: SimplePattern::Named(id, args),
                            location: Location::empty(),
                        };
                        result.push(pat);
                    }
                    for (index, arg) in args.iter().enumerate() {
                        let choices = self.generateChoices(arg);
                        for choice in choices {
                            let mut choiceArgs = Vec::new();
                            choiceArgs.extend(args.iter().cloned().take(index));
                            choiceArgs.push(choice);
                            choiceArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                            let id = Identifier {
                                name: name.toString(),
                                location: Location::empty(),
                            };
                            let pat = Pattern {
                                pattern: SimplePattern::Named(id, choiceArgs),
                                location: Location::empty(),
                            };
                            result.push(pat);
                        }
                    }
                }
                result
            }
            SimplePattern::Bind(_, _) => Vec::new(),
            SimplePattern::Tuple(args) => {
                let mut result = Vec::new();
                for (index, arg) in args.iter().enumerate() {
                    let choices = self.generateChoices(arg);
                    for choice in choices {
                        let mut choiceArgs = Vec::new();
                        choiceArgs.extend(args.iter().cloned().take(index));
                        choiceArgs.push(choice);
                        choiceArgs.extend(repeat(wildcardPattern.clone()).take(args.len() - index - 1));
                        let pat = Pattern {
                            pattern: SimplePattern::Tuple(choiceArgs),
                            location: Location::empty(),
                        };
                        result.push(pat);
                    }
                }
                result
            }
            SimplePattern::StringLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::IntegerLiteral(_) => {
                vec![wildcardPattern]
            }
            SimplePattern::Wildcard => Vec::new(),
        }
    }

    fn generateDecisions(
        &mut self,
        pattern: &Pattern,
        parentData: &DataPath,
        decision: &DecisionPath,
        mut bindings: Bindings,
    ) -> (DecisionPath, Bindings) {
        //println!("generateDecisions: {}, {}, {}", pattern, parentData, decision);
        match &pattern.pattern {
            SimplePattern::Named(origId, args) => {
                let name = self.resolver.moduleResolver.resolverName(&origId);
                if let Some(enumName) = self.resolver.variants.get(&name) {
                    let path = DataPath::Variant(Box::new(parentData.clone()), name, enumName.clone());
                    let mut decision = decision.add(path.clone());
                    for arg in args {
                        (decision, bindings) = self.generateDecisions(arg, &path, &decision, bindings);
                    }
                    (decision, bindings)
                } else {
                    (decision.add(DataPath::Class(Box::new(parentData.clone()), name)), bindings)
                }
            }
            SimplePattern::Bind(name, _) => {
                bindings.bindings.insert(decision.add(parentData.clone()), name.toString());
                (decision.add(DataPath::Wildcard(Box::new(parentData.clone()))), bindings)
            }
            SimplePattern::Tuple(args) => {
                let mut decision = decision.clone();
                let path = DataPath::Tuple(Box::new(parentData.clone()), args.len() as i64);
                decision = decision.add(path.clone());
                for (index, arg) in args.iter().enumerate() {
                    let path = DataPath::TupleIndex(Box::new(parentData.clone()), index as i64);
                    (decision, bindings) = self.generateDecisions(arg, &path, &decision, bindings);
                }
                (decision, bindings)
            }
            SimplePattern::StringLiteral(v) => (decision.add(DataPath::StringLiteral(Box::new(parentData.clone()), v.clone())), bindings),
            SimplePattern::IntegerLiteral(v) => (decision.add(DataPath::IntegerLiteral(Box::new(parentData.clone()), v.clone())), bindings),
            SimplePattern::Wildcard => (decision.add(DataPath::Wildcard(Box::new(parentData.clone()))), bindings),
        }
    }

    pub fn isMatch(&self, this: &Pattern, other: &Pattern) -> bool {
        match (&this.pattern, &other.pattern) {
            (SimplePattern::Named(id1, args1), SimplePattern::Named(id2, args2)) => {
                if id1 == id2 {
                    if args1.len() != args2.len() {
                        return false;
                    }
                    for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                        if !self.isMatch(arg1, arg2) {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            (SimplePattern::Wildcard, _) => true,
            (SimplePattern::Bind(_, _), _) => true,
            (SimplePattern::Tuple(args1), SimplePattern::Tuple(args2)) => {
                if args1.len() != args2.len() {
                    return false;
                }
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    if !self.isMatch(arg1, arg2) {
                        return false;
                    }
                }
                true
            }
            (SimplePattern::StringLiteral(val1), SimplePattern::StringLiteral(val2)) => val1 == val2,
            (SimplePattern::IntegerLiteral(val1), SimplePattern::IntegerLiteral(val2)) => val1 == val2,
            _ => false,
        }
    }

    // fn check(&mut self) -> Vec<Pattern> {
    //     let mut allChoices = BTreeSet::new();
    //     for branch in &self.branches {
    //         let branch = self.resolve(branch);
    //         allChoices.insert(branch.clone());
    //         //println!("Pattern {}", branch);
    //         let choices = self.generateChoices(&branch);
    //         for choice in choices {
    //             //println!("   Alt: {}", choice);
    //             allChoices.insert(choice);
    //         }
    //     }
    //     let mut remaining = allChoices.clone();
    //     for branch in self.branches.iter() {
    //         let resolvedBranch = self.resolve(branch);
    //         let mut reduced = BTreeSet::new();
    //         for m in &remaining {
    //             let isMatch = self.isMatch(&resolvedBranch, &m);
    //             //println!("{} ~ {} = {}", m, resolvedBranch, isMatch);
    //             if !isMatch {
    //                 reduced.insert(m.clone());
    //             }
    //         }
    //         if reduced.len() == remaining.len() {
    //             self.errors.push(ResolverError::RedundantPattern(branch.location.clone()));
    //         }
    //         remaining = reduced;
    //     }
    //     for m in remaining {
    //         self.errors.push(ResolverError::MissingPattern(m.to_string(), self.bodyLocation.clone()));
    //     }

    //     for err in &self.errors {
    //         err.reportOnly(self.resolver.ctx);
    //     }
    //     if !self.errors.is_empty() {
    //         std::process::exit(1);
    //     }
    //     allChoices.into_iter().collect()
    // }

    pub fn compile(&mut self) {
        let mut matches = Vec::new();

        for (index, branch) in self.branches.clone().iter().enumerate() {
            let branch = self.resolve(branch);
            let (decision, bindings) = self.generateDecisions(&branch, &DataPath::Root, &DecisionPath::new(), Bindings::new());
            let choices = self.generateChoices(&branch);
            matches.push(Match {
                kind: MatchKind::UserDefined(index as i64),
                pattern: branch,
                decisionPath: decision,
                bindings: bindings,
            });
            for choice in choices {
                //println!("   Alt: {}", choice);
                let (decision, bindings) = self.generateDecisions(&choice, &DataPath::Root, &DecisionPath::new(), Bindings::new());
                matches.push(Match {
                    kind: MatchKind::Alternative,
                    pattern: choice,
                    decisionPath: decision,
                    bindings: bindings,
                });
            }
        }

        let mut dataTypes = BTreeMap::new();
        for m in &matches {
            for path in &m.decisionPath.decisions {
                match path {
                    DataPath::Root => {}
                    DataPath::Tuple(parent, count) => {
                        dataTypes.insert(parent.clone(), DataType::Tuple(*count));
                    }
                    DataPath::TupleIndex(_, _) => {}
                    DataPath::Variant(parent, _, enumName) => {
                        dataTypes.insert(parent.clone(), DataType::Enum(enumName.clone()));
                    }
                    DataPath::IntegerLiteral(parent, _) => {
                        dataTypes.insert(parent.clone(), DataType::Integer);
                    }
                    DataPath::StringLiteral(parent, _) => {
                        dataTypes.insert(parent.clone(), DataType::String);
                    }
                    DataPath::Class(parent, name) => {
                        dataTypes.insert(parent.clone(), DataType::Class(name.clone()));
                    }
                    DataPath::Wildcard(_) => {}
                }
            }
        }
        for (path, ty) in &dataTypes {
            println!("{} {}", path, ty);
        }
        for m in &matches {
            println!("Decision {}", m.decisionPath);
        }
        let mut pendingPaths = Vec::new();
        pendingPaths.push(DataPath::Root);
        let mut node = self.buildNode(pendingPaths, &DecisionPath::new(), &dataTypes, &matches);
        node.add(&matches);
    }

    fn buildNode(
        &mut self,
        mut pendingPaths: Vec<DataPath>,
        currentDecision: &DecisionPath,
        dataTypes: &BTreeMap<Box<DataPath>, DataType>,
        allMatches: &Vec<Match>,
    ) -> Node {
        let currentPath = pendingPaths.remove(0);
        if let Some(ty) = dataTypes.get(&currentPath) {
            //println!("Building node for {}, {} / [{}]", currentPath, ty, currentDecision);
            match ty {
                DataType::Class(_) => todo!(),
                DataType::Enum(enumName) => {
                    let e = self.resolver.enums.get(enumName).expect("enumName not found");
                    let mut cases = BTreeMap::new();
                    for variant in &e.variants {
                        let casePath = DataPath::Variant(Box::new(currentPath.clone()), variant.name.clone(), enumName.clone());
                        let currentDecision = currentDecision.add(casePath.clone());
                        let pendings = pendingPaths.clone();
                        pendingPaths.insert(0, casePath.clone());
                        let node = self.buildNode(pendings, &currentDecision, dataTypes, allMatches);
                        cases.insert(Case::Variant(variant.name.clone()), node);
                    }
                    let switch = Switch { cases: cases };
                    Node::Switch(switch)
                }
                DataType::Tuple(size) => {
                    let currentDecision = currentDecision.add(DataPath::Tuple(Box::new(currentPath.clone()), *size));
                    let mut pendings = Vec::new();
                    for index in 0..*size {
                        let argPath = DataPath::TupleIndex(Box::new(currentPath.clone()), index);
                        pendings.insert(0, argPath);
                    }
                    pendings.reverse();
                    pendings.extend(pendingPaths.clone());
                    let node = self.buildNode(pendings, &currentDecision, dataTypes, allMatches);
                    let tuple = Tuple { next: Box::new(node) };
                    Node::Tuple(tuple)
                }
                DataType::Integer => {
                    let mut cases = BTreeMap::new();
                    let mut values = BTreeSet::new();
                    for m in allMatches {
                        if m.decisionPath.decisions.starts_with(&currentDecision.decisions[..]) {
                            if m.decisionPath.decisions.len() > currentDecision.decisions.len() + 1 {
                                match &m.decisionPath.decisions[currentDecision.decisions.len()] {
                                    DataPath::IntegerLiteral(_, value) => {
                                        values.insert(value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    for value in values {
                        let path = DataPath::IntegerLiteral(Box::new(currentPath.clone()), value.clone());
                        let mut pendingPaths = pendingPaths.clone();
                        pendingPaths.insert(0, path.clone());
                        let currentDecision = &currentDecision.add(path);
                        let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                        cases.insert(Case::Integer(value.clone()), node);
                    }
                    let path = DataPath::Wildcard(Box::new(currentPath.clone()));
                    let mut pendingPaths = pendingPaths.clone();
                    pendingPaths.insert(0, path.clone());
                    let currentDecision = &currentDecision.add(path);
                    let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                    cases.insert(Case::Default, node);
                    let switch = Switch { cases: cases };
                    Node::Switch(switch)
                }
                DataType::String => {
                    let mut cases = BTreeMap::new();
                    let mut values = BTreeSet::new();
                    for m in allMatches {
                        if m.decisionPath.decisions.starts_with(&currentDecision.decisions[..]) {
                            if m.decisionPath.decisions.len() >= currentDecision.decisions.len() + 1 {
                                match &m.decisionPath.decisions[currentDecision.decisions.len()] {
                                    DataPath::StringLiteral(_, value) => {
                                        values.insert(value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    for value in values {
                        let path = DataPath::StringLiteral(Box::new(currentPath.clone()), value.clone());
                        let mut pendingPaths = pendingPaths.clone();
                        pendingPaths.insert(0, path.clone());
                        let currentDecision = &currentDecision.add(path);
                        let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                        cases.insert(Case::Integer(value.clone()), node);
                    }
                    let path = DataPath::Wildcard(Box::new(currentPath.clone()));
                    let mut pendingPaths = pendingPaths.clone();
                    pendingPaths.insert(0, path.clone());
                    let currentDecision = &currentDecision.add(path);
                    let node = self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches);
                    cases.insert(Case::Default, node);
                    let switch = Switch { cases: cases };
                    Node::Switch(switch)
                }
            }
        } else {
            if pendingPaths.is_empty() {
                let end = End {
                    decisionPath: currentDecision.clone(),
                };
                return Node::End(end);
            }
            self.buildNode(pendingPaths, currentDecision, dataTypes, allMatches)
        }
    }
}

#[derive(Clone)]
struct Tuple {
    next: Box<Node>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Case {
    Variant(QualifiedName),
    Integer(String),
    String(String),
    Default,
}

#[derive(Clone)]
struct Switch {
    cases: BTreeMap<Case, Node>,
}

#[derive(Clone)]
struct Bind {
    var: InstructionId,
    name: String,
    blockId: BlockId,
}

#[derive(Clone)]
struct End {
    decisionPath: DecisionPath,
}

#[derive(Clone)]
enum Node {
    Tuple(Tuple),
    Switch(Switch),
    Bind(Bind),
    End(End),
}

impl Node {
    fn add(&mut self, matches: &Vec<Match>) {
        match self {
            Node::Tuple(tuple) => tuple.next.add(matches),
            Node::Switch(switch) => {
                for (_, node) in &mut switch.cases {
                    node.add(matches);
                }
            }
            Node::Bind(_) => todo!(),
            Node::End(end) => {
                let mut localMatch: Option<Match> = None;
                for m in matches {
                    let matchResult = matchDecisions(end.decisionPath.clone(), m.decisionPath.clone());
                    if matchResult {
                        //println!("MATCH end {} //// {}", end.decisionPath, m.decisionPath);
                        match &localMatch {
                            Some(local) => match (&local.kind, &m.kind) {
                                (MatchKind::Alternative, MatchKind::UserDefined(_)) => {
                                    localMatch = Some(m.clone());
                                }
                                (MatchKind::UserDefined(i1), MatchKind::UserDefined(i2)) => {
                                    if *i2 < *i1 {
                                        localMatch = Some(m.clone());
                                    }
                                }
                                _ => {}
                            },
                            None => {
                                localMatch = Some(m.clone());
                            }
                        }
                    }
                }
                if let Some(m) = localMatch {
                    println!("FINAL MATCH {} for {}, bindings: {}", end.decisionPath, m.kind, m.bindings);
                }
            }
        }
    }
}

fn matchDecisions(mut nodeDecisionPath: DecisionPath, mut matchDecisionPath: DecisionPath) -> bool {
    loop {
        if matchDecisionPath.decisions.is_empty() {
            return nodeDecisionPath.decisions.is_empty();
        }
        let path = matchDecisionPath.decisions.remove(0);
        nodeDecisionPath = removePaths(&path, nodeDecisionPath);
    }
}

fn removePaths(path: &DataPath, mut nodeDecisionPath: DecisionPath) -> DecisionPath {
    loop {
        if nodeDecisionPath.decisions.is_empty() {
            break;
        }
        let nodePath = &nodeDecisionPath.decisions[0];
        let remove = match (path, nodePath) {
            (DataPath::Wildcard(parent), _) => nodePath.isChild(parent),
            (p1, p2) => p1 == p2,
        };
        if remove {
            nodeDecisionPath.decisions.remove(0);
        } else {
            break;
        }
    }
    nodeDecisionPath
}

#[derive(Clone)]
enum MatchKind {
    UserDefined(i64),
    Alternative,
}

impl fmt::Display for MatchKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MatchKind::UserDefined(value) => write!(f, "UserDefined({})", value),
            MatchKind::Alternative => write!(f, "Alternative"),
        }
    }
}

#[derive(Clone)]
struct Match {
    kind: MatchKind,
    pattern: Pattern,
    decisionPath: DecisionPath,
    bindings: Bindings,
}

#[derive(Clone)]
struct Bindings {
    bindings: BTreeMap<DecisionPath, String>,
}

impl Bindings {
    pub fn new() -> Bindings {
        Bindings { bindings: BTreeMap::new() }
    }
}

impl fmt::Display for Bindings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bindings {{ ")?;
        for (key, value) in &self.bindings {
            write!(f, "{}: {}, ", key, value)?;
        }
        write!(f, "}}")
    }
}
