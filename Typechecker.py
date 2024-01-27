import Syntax
import IR

class Substitution(object):
    def __init__(self):
        self.substitutions = {}

    def add(self, var, type):
        self.substitutions[var] = type

    def apply(self, ty):
        res = ty
        while True:
            if isinstance(res, TypeVar):
                if res in self.substitutions:
                    res = self.substitutions[res]
                else:
                    return res
            else:
                return res

class NamedType(object):
    def __init__(self):
        self.value = None

    def __str__(self):
        return "Named:%s" % self.value

class TypeVar(object):
    def __init__(self):
        self.value = None
    
    def __str__(self):
        return "$tv.%s" % self.value

class Typechecker(object):
    def __init__(self):
        self.substitution = Substitution()
        self.nextVar = 0
        self.types = {}

    def getNextVar(self):
        v = TypeVar()
        v.value = self.nextVar
        self.nextVar += 1
        return v

    def initialize(self, fn):
        for arg in fn.args:
            namedType = NamedType()
            namedType.value = arg.type.name
            self.types[arg.name] = namedType
        for i in fn.body.instructions:
            if isinstance(i, IR.BlockBegin):
                continue
            if isinstance(i, IR.BlockEnd):
                continue
            if isinstance(i, IR.Bind):
                v = self.getNextVar()
                self.types[i.name] = v
            v = self.getNextVar()
            self.types[i.id] = v
            #print("Initializing %s = %s" % (i.id, v))

    def unify(self, type1, type2):
        #print("Unifying %s/%s" % (type1, type2))
        type1 = self.substitution.apply(type1)
        type2 = self.substitution.apply(type2)
        print("Unifying2 %s/%s" % (type1, type2))
        if isinstance(type1, TypeVar):
            self.substitution.add(type1, type2)
        elif isinstance(type2, TypeVar):
            self.substitution.add(type2, type1)

    def check(self, fn):
        unitType = NamedType()
        unitType.value = "Main.Unit"
        boolType = NamedType()
        boolType.value = "Main.Bool"
        print("Type checking %s" % fn.name)
        self.initialize(fn)
        for i in fn.body.instructions:
            if isinstance(i, IR.BlockBegin):
                continue
            if isinstance(i, IR.BlockEnd):
                continue
            if isinstance(i, IR.NamedFunctionCall):
                #print("Checking function call for %s" % i.name)
                #print("%s" % i.name.item.return_type.name)
                returnType = NamedType()
                returnType.value = i.name.item.return_type.name
                self.unify(self.types[i.id], returnType)
            elif isinstance(i, IR.Bind):
                self.unify(self.types[i.name], self.types[i.rhs])
                self.unify(self.types[i.id], unitType)
            elif isinstance(i, IR.VarRef):
                self.unify(self.types[i.id], self.types[i.name])
            elif isinstance(i, IR.BoolLiteral):
                self.unify(self.types[i.id], boolType)
            elif isinstance(i, IR.If):
                self.unify(self.types[i.cond], boolType)
                self.unify(self.types[i.id], self.types[i.true_branch])
                self.unify(self.types[i.id], self.types[i.false_branch])
            else:
                print("Not handled", type(i))


def checkFunction(f):
    checker = Typechecker()
    checker.check(f)

def checkProgram(program):
    for m in program.modules:
        for item in m.items:
            if isinstance(item, Syntax.Function):
                checkFunction(item)
            if isinstance(item, Syntax.Class):
                for m in item.methods:
                    checkFunction(m)