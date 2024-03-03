import Compiler.Token as Token

def isNumber(c):
    return c >= '0' and c <= '9'

def isLetter(c):
    return (c >= 'a' and c <= 'z') or (c >= 'A' and c <= 'Z')

class Lexer(object):
    def __init__(self):
        self.tokens = []
        self.index = 0
        self.current = ""

    def endToken(self):
        if self.current:
            match self.current:
                case "module":
                    self.addToken(Token.Keyword(self.current))
                case "fn":
                    self.addToken(Token.Keyword(self.current))
                case "class":
                    self.addToken(Token.Keyword(self.current))
                case "extern":
                    self.addToken(Token.Keyword(self.current))
                case "enum":
                    self.addToken(Token.Keyword(self.current))
                case "import":
                    self.addToken(Token.Keyword(self.current))
                case "as":
                    self.addToken(Token.Keyword(self.current))
                case "return":
                    self.addToken(Token.Keyword(self.current))
                case "let":
                    self.addToken(Token.Keyword(self.current))
                case "if":
                    self.addToken(Token.Keyword(self.current))
                case "else":
                    self.addToken(Token.Keyword(self.current))
                case "loop":
                    self.addToken(Token.Keyword(self.current))
                case "break":
                    self.addToken(Token.Keyword(self.current))
                case "continue":
                    self.addToken(Token.Keyword(self.current))
                case "return":
                    self.addToken(Token.Keyword(self.current))
                case "false":
                    self.addToken(Token.Keyword(self.current))
                case "true":
                    self.addToken(Token.Keyword(self.current))
                case "derive":
                    self.addToken(Token.Keyword(self.current))
                case "trait":
                    self.addToken(Token.Keyword(self.current))
                case "mut":
                    self.addToken(Token.Keyword(self.current))
                case "for":
                    self.addToken(Token.Keyword(self.current))
                case "in":
                    self.addToken(Token.Keyword(self.current))
                case "instance":
                    self.addToken(Token.Keyword(self.current))
                case "match":
                    self.addToken(Token.Keyword(self.current))
                case "and":
                    self.addToken(Token.And())
                case "or":
                    self.addToken(Token.Or())
                case "_":
                    self.addToken(Token.Wildcard())
                case _:
                    if self.current[0].isupper():
                        self.addToken(Token.TypeIdentifier(self.current))
                    else:
                        self.addToken(Token.VarIdentifier(self.current))
            self.current = ""

    def addToken(self, token):
        self.tokens.append(token)

    def step(self):
        self.index = self.index + 1

    def lex(self, chars):
        while True:
            if len(chars) <= self.index:
                return self.tokens
            current = chars[self.index]
            match current:
                case '\n':
                    self.endToken()
                    self.step()
                case '.':
                    self.endToken()
                    self.addToken(Token.Dot())
                    self.step()
                case '(':
                    self.endToken()
                    self.addToken(Token.LeftParen())
                    self.step()
                case ')':
                    self.endToken()
                    self.addToken(Token.RightParen())
                    self.step()
                case '{':
                    self.endToken()
                    self.addToken(Token.LeftCurly())
                    self.step()
                case '}':
                    self.endToken()
                    self.addToken(Token.RightCurly())
                    self.step()
                case '[':
                    self.endToken()
                    self.addToken(Token.LeftBracket())
                    self.step()
                case ']':
                    self.endToken()
                    self.addToken(Token.RightBracket())
                    self.step()
                case ';':
                    self.endToken()
                    self.addToken(Token.Semicolon())
                    self.step()
                case ',':
                    self.endToken()
                    self.addToken(Token.Comma())
                    self.step()
                case '*':
                    self.endToken()
                    self.addToken(Token.Mul())
                    self.step()
                case '/':
                    self.endToken()
                    self.addToken(Token.Div())
                    self.step()
                case ':':
                    self.endToken()
                    self.addToken(Token.Colon())
                    self.step()
                case '"':
                    self.endToken()
                    self.step()
                    s = ""
                    while len(chars) > self.index:
                        c = chars[self.index]
                        if c == '"':
                            self.step()
                            break
                        else:
                            s += c
                        self.step()
                    self.addToken(Token.String(s))
                case '=':
                    self.endToken()
                    if chars[self.index + 1] == '>':
                        self.addToken(Token.RightDoubleArrow())
                        self.step()
                    elif chars[self.index + 1] == '=':
                        self.addToken(Token.DoubleEqual())
                        self.step()
                    else:    
                        self.addToken(Token.Equal())
                    self.step()
                case '@':
                    self.endToken()
                    self.addToken(Token.At())
                    self.step()
                case '+':
                    self.endToken()
                    self.addToken(Token.Plus())
                    self.step()
                case '>':
                    self.endToken()
                    if chars[self.index + 1] == '=':
                        self.addToken(Token.GreaterThanOrEqual())
                        self.step()
                    else:
                        self.addToken(Token.GreaterThan())
                    self.step()
                case '<':
                    self.endToken()
                    if chars[self.index + 1] == '=':
                        self.addToken(Token.LessThanOrEqual())
                        self.step()
                    else:
                        self.addToken(Token.LessThan())
                    self.step()
                case '!':
                    self.endToken()
                    if chars[self.index + 1] == '=':
                        self.addToken(Token.NotEqual())
                        self.step()
                    else:
                        self.addToken(Token.ExclamationMark())
                    self.step()
                case '-':
                    self.endToken()
                    if chars[self.index + 1] == '>':
                        self.addToken(Token.RightArrow())
                        self.step()
                    else:    
                        self.addToken(Token.Minus())
                    self.step()
                case ' ':
                    self.endToken()
                    self.step()
                case _:
                    if isLetter(current):
                        self.current += current
                        self.step()
                    elif isNumber(current):
                        self.current += current
                        self.step()
                    elif current == '_':
                        self.current += current
                        self.step()
                    else:
                        print("Unsupported character '%s'" % current)
                        self.step()