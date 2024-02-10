import sys

class QualifiedName(object):
    def __init__(self, moduleName, name, className = None):
        self.moduleName = moduleName
        self.name = name
        self.className = className

    def __str__(self):
        if self.className:
            return "%s.%s.%s" % (self.moduleName, self.className, self.name)
        else:
            return "%s.%s" % (self.moduleName, self.name)

    def __eq__(self, other):
        return self.moduleName == other.moduleName and self.name == other.name and self.className == other.className

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return self.name.__hash__()
    
def error(msg):
    print(msg)
    sys.exit(1)

def getBool():
    name = QualifiedName("Bool", "Bool")
    return name

def getUnit():
    name = QualifiedName("Unit", "Unit")
    return name
