import Compiler.IR as IR
import Compiler.Util as Util
import Compiler.DependencyProcessor as DependencyProcessor
import Compiler.Ownership.DataFlowDependency as DataFlowDependency
import Compiler.Ownership.MemberInfo as MemberInfo
import Compiler.Ownership.Allocator as Allocator

class Value(object):
    def __init__(self):
        self.source = None

    def __str__(self):
        return "(Value/%s)" % (self.source)

    def __repr__(self) -> str:
        return self.__str__()

    def normalize(self):
        return (self, False)

    def isValid(self):
        return True

    def buildSource(self, arg, allocator):
        return []

class FieldAccess(object):
    def __init__(self, receiver, index):
        self.receiver = receiver
        self.index = index

    def __str__(self):
        return "(%s.%s)" % (self.receiver, self.index)

    def __repr__(self) -> str:
        return self.__str__()

    def normalize(self):
        if isinstance(self.receiver, Record):
            if self.receiver.index == self.index:
                return (self.receiver.value, True)
        (value, normalized) = self.receiver.normalize()
        self.receiver = value
        return (self, normalized)

    def isValid(self):
        if isinstance(self.receiver, Record):
            if self.receiver.index == self.index:
                return True
            else:
                return False
        else:
            return True

    def buildSource(self, arg, allocator):
        member = MemberInfo.MemberInfo()
        members = self.receiver.buildSource(arg, allocator)
        if isinstance(self.receiver, Value):
            member.root = arg.group_var
        else:
            member.root = members[-1].info.group_var
        member.kind = MemberInfo.MemberKind()
        member.kind.index = self.index
        member.kind.type = MemberInfo.FieldKind
        member.info = allocator.nextTypeVariableInfo()
        members.append(member)
        return members

class Record(object):
    def __init__(self, value, index):
        self.value = value
        self.index = index
    
    def __str__(self):
        return "record(%s/%s)" % (self.value, self.index)
    
    def __repr__(self) -> str:
        return self.__str__()
    
    def normalize(self):
        (value, normalized) = self.value.normalize()
        self.value = value
        return (self, normalized)

    def isValid(self):
        return self.value.isValid()

class InferenceEngine(object):
    def __init__(self):
        self.fn = None

    def inferFn(self, fn):
        self.fn = fn
        #print("DataFlowPath for %s" % fn.name)
        return self.createPaths()

    def processPath(self, path):
        root = self.fn.body.getInstruction(path[0])
        value = Value()
        if isinstance(root, IR.ValueRef):
            value.source = root.name.value
        prev = None
        for p in path:
            instruction = self.fn.body.getInstruction(p)
            if isinstance(instruction, IR.Bind):
                pass
            elif isinstance(instruction, IR.MemberAccess):
                value = FieldAccess(value, instruction.index)
            elif isinstance(instruction, IR.ValueRef):
                for i in instruction.indices:
                    value = FieldAccess(value, i)
            elif isinstance(instruction, IR.NamedFunctionCall):
                if instruction.ctor:
                    for (arg_index, arg) in enumerate(instruction.args):
                        if arg == prev:
                            value = Record(value, arg_index)
            elif isinstance(instruction, IR.If):
                pass
            elif isinstance(instruction, IR.BlockRef):
                pass
            else:
                print("Processing path element %s %s %s" % (p, instruction, type(instruction)))
            prev = p
        return value

    def createPaths(self):
        arg_instructions = []
        end_instruction = self.fn.body.getFirst().getLastReal()
        all_dependencies = DataFlowDependency.getDataFlowDependencies(self.fn)
        paths = {}
        for block in self.fn.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.ValueRef):
                    if i.name.arg:
                        arg_instructions.append(i.id)
        groups = DependencyProcessor.processDependencies(all_dependencies)
        for g in groups:
            for item in g.items:
                item_paths = []
                deps = all_dependencies[item]
                if len(deps) == 0:
                    item_paths = [[item]]
                else:
                    for dep in deps:
                        if dep in g.items:
                            continue
                        dep_paths = paths[dep]
                        for dep_path in dep_paths:
                            item_paths.append(dep_path + [item])
                paths[item] = item_paths
        final_paths = []
        for (i, paths) in paths.items():
            for path in paths:
                if path[0] in arg_instructions:
                    if path[-1] == end_instruction.id:
                        #print("root %s" % i)
                        #print("path", path)
                        path = self.processPath(path)
                        #print("processed path", path)
                        #print("processed path is %s" % path.isValid())
                        more = True
                        while more:
                            (path, more) = path.normalize()
                        #print("Normalized", path)
                        #print("Normalized", path.isValid())
                        if path.isValid():
                            path = splitPath(path, Allocator.Allocator())
                            final_paths.append(path)
        return final_paths

class DataFlowPath(object):
    def __init__(self, arg, result, src, dest):
        self.arg = arg
        self.result = result
        self.src = src
        self.dest = dest

    def __str__(self):
        return "path(%s/%s/%s/%s)" % (self.arg, self.result, self.src, self.dest)

    def __repr__(self) -> str:
        return self.__str__()

def splitPath(path, allocator):
    arg = allocator.nextTypeVariableInfo()
    result = allocator.nextTypeVariableInfo()
    dest_members = []
    while True:
        if isinstance(path, Record):
            member = MemberInfo.MemberInfo()
            if len(dest_members) == 0: # first
                member.root = result.group_var
            else:
                member.root = dest_members[-1].info.group_var
            member.kind = MemberInfo.MemberKind()
            member.kind.index = path.index
            member.kind.type = MemberInfo.FieldKind
            member.info = allocator.nextTypeVariableInfo()
            dest_members.append(member)
            path = path.value
        else:
            break
    src_members = path.buildSource(arg, allocator)
    return DataFlowPath(arg, result, src_members, dest_members)

def infer(f):
    engine = InferenceEngine()
    return engine.inferFn(f)