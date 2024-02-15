import IR
import Util
import DependencyProcessor

def getDepsForInstruction(i, fn):
    if isinstance(i, IR.ValueRef):
        return [i.bind_id]
    elif isinstance(i, IR.VarRef):
        if i.name.arg:
            return []
        else:
            return [i.bind_id]
    elif isinstance(i, IR.Bind):
        return [i.rhs]
    elif isinstance(i, IR.BlockRef):
        b = fn.body.getBlock(i.value)
        return [b.getLastReal().id]
    elif isinstance(i, IR.NamedFunctionCall):
        return i.args
    elif isinstance(i, IR.DropVar):
        return []
    elif isinstance(i, IR.Converter):
        return [i.arg]
    elif isinstance(i, IR.BoolLiteral):
        return []
    elif isinstance(i, IR.Nop):
        return []
    elif isinstance(i, IR.If):
        true_branch = fn.body.getBlock(i.true_branch)
        false_branch = fn.body.getBlock(i.false_branch)
        t_id = true_branch.getLast().id
        f_id = false_branch.getLast().id
        return [t_id, f_id]
    else:
        Util.error("OI: getDepsForInstruction not handling %s %s" % (type(i), i))

class Value(object):
    def __init__(self):
        pass

    def __str__(self):
        return "Value"

    def normalize(self):
        return (self, False)

    def isValid(self):
        return True

class FieldAccess(object):
    def __init__(self, receiver, index):
        self.receiver = receiver
        self.index = index

    def __str__(self):
        return "(%s.%s)" % (self.receiver, self.index)

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

class Record(object):
    def __init__(self, value, index):
        self.value = value
        self.index = index
    
    def __str__(self):
        return "record(%s/%s)" % (self.value, self.index)
    
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
        print("DataFlowPath for %s" % fn.name)
        self.createPaths()

    def processPath(self, path):
        value = Value()
        prev = None
        for p in path:
            instruction = self.fn.body.getInstruction(p)
            if isinstance(instruction, IR.VarRef):
                pass
            elif isinstance(instruction, IR.Bind):
                pass
            elif isinstance(instruction, IR.MemberAccess):
                value = FieldAccess(value, instruction.index)
            elif isinstance(instruction, IR.ValueRef):
                for i in instruction.indices:
                    value = FieldAccess(value, i)
            elif isinstance(instruction, IR.Converter):
                pass
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
        all_dependencies = {}
        paths = {}
        for block in self.fn.body.blocks:
            for i in block.instructions:
                if isinstance(i, IR.VarRef):
                    if i.name.arg:
                        arg_instructions.append(i.id)
                all_dependencies[i.id] = getDepsForInstruction(i, self.fn)
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
                            final_paths.append(path)
        return final_paths

def infer(program):
    for f in program.functions.values():
        engine = InferenceEngine()
        engine.inferFn(f)