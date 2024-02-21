import Compiler.Ownership.Signatures as Signatures
import Compiler.Ownership.Inference as Inference
import Compiler.Ownership.MemberInfo as MemberInfo
import Compiler.Ownership.Allocator as Allocator
import Compiler.Ownership.TypeVariableInfo as TypeVariableInfo
import copy

class Normalizer(object):
    def __init__(self):
        self.allocator = Allocator.Allocator()
        self.ownership_vars = {}
        self.group_vars = {}

    def normalizeOwnershipVar(self, var):
        if var in self.ownership_vars:
            ownership_var = self.ownership_vars[var]
        else:
            ownership_var = self.allocator.nextOwnershipVar()
            self.ownership_vars[var] = ownership_var
        return ownership_var
        
    def normalizeGroupVar(self, var):
        if var in self.group_vars:
            group_var = self.group_vars[var]
        else:
            group_var = self.allocator.nextGroupVar()
            self.group_vars[var] = group_var
        return group_var

    def normalize(self, info):
        res = copy.deepcopy(info)
        res.ownership_var = self.normalizeOwnershipVar(info.ownership_var)
        res.group_var = self.normalizeGroupVar(info.group_var)
        return res

def filterOutBorrowingMembers(signature, ownership_dep_map, members, ownerships):
    #print("Signature", signature)
    #print("ownership_dep_map", ownership_dep_map)
    #print("members", members)
    relevant_members = []
    for arg in signature.args:
        if arg.group_var in ownership_dep_map:
            ownership_vars = ownership_dep_map[arg.group_var]
            #print("Arg", arg)
            #print("vars", ownership_vars)
            for member in members:
                if member.info.ownership_var in ownership_vars:
                    #print("member is relevant", member)
                    relevant_members.append(member)
    #print("relevant_members", relevant_members)
    borrows = []
    for member in relevant_members:
        if isinstance(ownerships[member.info.ownership_var], Inference.Borrow):
            borrows.append(member.info.ownership_var)
    #print("Borrows", borrows)
    only_borrowing_members = []
    for member in relevant_members:
        ownership_vars = ownership_dep_map[member.info.group_var]
        containsBorrow = member.info.ownership_var in borrows
        for o in ownership_vars:
            if o in borrows:
                containsBorrow = True
                break
        if containsBorrow:
            only_borrowing_members.append(member)
    #print("only_borrowing_members", only_borrowing_members)
    return (only_borrowing_members, borrows)

def collectChildMembers(normalizer, var, members):
    def sortFunc(member):
        return member.kind.index
    children = []
    for member in members:
        if member.root == var:
            children.append(member)
    children.sort(key=sortFunc)
    normalized_children = []
    for child in children:
        normalized_child = copy.deepcopy(child)
        normalized_child.root = normalizer.normalizeGroupVar(child.root)
        normalized_child.info = normalizer.normalize(child.info)
        normalized_children.append(normalized_child)
    for child in children:
        normalized_children += collectChildMembers(normalizer, child.info.group_var, members)
    return normalized_children

def normalizeFunctionOwnershipSignature(signature, ownership_dep_map, members, ownerships):
    normalizer = Normalizer()
    #print("Signature", signature)
    #print("ownership_dep_map", ownership_dep_map)
    #print("members", members)
    (only_borrowing_members, borrows) = filterOutBorrowingMembers(signature, ownership_dep_map, members, ownerships)
    ordered_members = []
    normalized_args = []
    for arg in signature.args:
        normalized_args.append(normalizer.normalize(arg))
    normalized_result = normalizer.normalize(signature.result)
    for arg in signature.args:
        ordered_members += collectChildMembers(normalizer, arg.group_var, only_borrowing_members)
    #print("Ordered members", ordered_members)
    signature.args = normalized_args
    signature.members = ordered_members
    signature.result = normalized_result
    signature.allocator = normalizer.allocator
    signature.borrows = borrows
    #print("Signature2", signature)
    return signature