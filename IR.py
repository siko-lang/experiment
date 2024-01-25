import Syntax

class TempVar(object):
    def __init__(self):
        self.value = 0

    def __str__(self):
        return "$tmp_%s" % self.value

class InstructionId(object):
    def __init__(self):
        self.value = 0

    def __str__(self):
        return "$%s" % self.value

class BaseInstruction(object):
    def __init__(self):
        self.id = 0

class BlockBegin(BaseInstruction):
    def __init__(self):
        super().__init__()

    def __str__(self):
        return "<block begin>"

class BlockEnd(BaseInstruction):
    def __init__(self):
        super().__init__()

    def __str__(self):
        return "<block end>"

class NamedFunctionCall(BaseInstruction):
    def __init__(self):
        self.name = None
        self.args = []

    def __str__(self):
        return "%s(%s)" % (self.name, ", ".join(self.args))

class DynamicFunctionCall(BaseInstruction):
    def __init__(self):
        self.callable = None
        self.args = []

    def __str__(self):
        return "%s(%s)" % (self.callable, ", ".join(self.args))

class MethodCall(BaseInstruction):
    def __init__(self):
        self.receiver = None
        self.name = None
        self.args = []

    def __str__(self):
        return "%s.%s(%s)" % (self.receiver, self.name, ", ".join(self.args))

class Bind(BaseInstruction):
    def __init__(self):
        self.name = None
        self.rhs = None

    def __str__(self):
        return "%s = %s" % (self.name, self.rhs)

class MemberAccess(BaseInstruction):
    def __init__(self):
        self.receiver = None
        self.name = None
        
    def __str__(self):
        return "%s.%s" % (self.receiver, self.name)

class VarRef(BaseInstruction):
    def __init__(self):
        self.name = None
        
    def __str__(self):
        return "%s" % (self.name)

class If(BaseInstruction):
    def __init__(self):
        self.cond = None
        self.true_branch = None
        self.false_branch = None

    def __str__(self):
        if self.false_branch:
            return "if %s then %s else %s" % (self.cond, self.true_branch, self.false_branch)
        else:
            return "if %s then %s" % (self.cond, self.true_branch)

class Loop(BaseInstruction):
    def __init__(self):
        self.var = None
        self.init = None
        self.body = None

    def __str__(self):
        return "loop %s = %s %s" % (self.var, self.init, self.body)

class Break(BaseInstruction):
    def __init__(self):
        self.arg = None

    def __str__(self):
        return "break %s" % self.arg

class Continue(BaseInstruction):
    def __init__(self):
        self.arg = None
    
    def __str__(self):
        return "continue %s" % self.arg

class Return(BaseInstruction):
    def __init__(self):
        self.arg = None
    
    def __str__(self):
        return "return %s" % self.arg

class Body(object):
    def __init__(self):
        self.instructions = []

    def dump(self):
        for (index, i) in enumerate(self.instructions):
           print("$%d. %s" % (index, i))

class Processor(object):
    def __init__(self):
        self.instructions = []

    def addInstruction(self, instruction):
        index = len(self.instructions)
        id = InstructionId()
        id.value = index
        instruction.id = id
        self.instructions.append(instruction)
        return id

    def processArgs(self, eargs):
        args = []
        for arg in eargs:
            args.append(self.processExpr(arg))
        args = map(lambda x: str(x), args)
        return args

    def processExpr(self, expr):
        if isinstance(expr, Syntax.Block):
            self.addInstruction(BlockBegin())
            last = None
            for s in expr.statements:
                last = self.processExpr(s)
            self.addInstruction(BlockEnd())
            return last
        elif isinstance(expr, Syntax.LetStatement):
            id = self.processExpr(expr.rhs)
            bind = Bind()
            bind.name = expr.var_name
            bind.rhs = id
            return self.addInstruction(bind)
        elif isinstance(expr, Syntax.ExprStatement):
            return self.processExpr(expr.expr)
        elif isinstance(expr, Syntax.MemberCall):
            receiver = self.processExpr(expr.receiver)
            args = self.processArgs(expr.args)
            call = MethodCall()
            call.receiver = receiver
            call.name = expr.name
            call.args = args
            return self.addInstruction(call)
        elif isinstance(expr, Syntax.FunctionCall):
            args = self.processArgs(expr.args)
            if isinstance(expr.id, Syntax.VarRef):
                call = NamedFunctionCall()
                call.name = expr.id.name
                call.args = args
                return self.addInstruction(call)
            else:
                id = self.processExpr(expr.id)
                call = DynamicFunctionCall()
                call.callable = id
                call.args = args
                return self.addInstruction(call)
        elif isinstance(expr, Syntax.MemberAccess):
            receiver = self.processExpr(expr.receiver)
            access = MemberAccess()
            access.receiver = receiver
            access.name = expr.name
            return self.addInstruction(access)
        elif isinstance(expr, Syntax.VarRef):
            ref = VarRef()
            ref.name = expr.name
            return self.addInstruction(ref)
        elif isinstance(expr, Syntax.If):
            if_instr = If()
            if_instr.cond = self.processExpr(expr.cond)
            if_instr.true_branch = self.processExpr(expr.true_branch)
            if expr.false_branch:
                if_instr.false_branch = self.processExpr(expr.false_branch)
            return self.addInstruction(if_instr)
        elif isinstance(expr, Syntax.Loop):
            init = self.processExpr(expr.init)
            body = self.processExpr(expr.body)
            loop = Loop()
            loop.var = expr.var
            loop.init = init
            loop.body = body
            return self.addInstruction(loop)
        elif isinstance(expr, Syntax.Break):
            arg = self.processExpr(expr.arg)
            br = Break()
            br.arg = arg
            return self.addInstruction(br)
        elif isinstance(expr, Syntax.Continue):
            arg = self.processExpr(expr.arg)
            cont = Continue()
            cont.arg = arg
            return self.addInstruction(cont)
        elif isinstance(expr, Syntax.Return):
            arg = self.processExpr(expr.arg)
            ret = Return()
            ret.arg = arg
            return self.addInstruction(ret)
        else:
            print("Expr not handled", type(expr))

def convertProgram(program):
    for m in program.modules:
        #print("Processing module %s" % m.name)
        for item in m.items:
            if isinstance(item, Syntax.Function):
                fn = item
                #print("Processing fn %s" % fn.name)
                processor = Processor()
                processor.processExpr(fn.body)
                body = Body()
                body.instructions = processor.instructions
                fn.body = body
                #fn.body.dump()

                