use law_of_demeter_core::domain::type_shape::TypeRef;

pub(crate) fn field_type(text: &str) -> Result<TypeRef, String> {
    let mut parser = Parser::new(text);
    let type_ref = parser.type_ref()?;
    parser.finish()?;
    Ok(type_ref)
}

pub(crate) struct TypeParameter {
    pub(crate) name: String,
    pub(crate) bounds: Vec<TypeRef>,
}

pub(crate) struct MethodSignature {
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) parameters: Vec<TypeRef>,
    pub(crate) returns: Option<TypeRef>,
}

pub(crate) fn method_signature(text: &str) -> Result<MethodSignature, String> {
    let mut parser = Parser::new(text);
    let type_parameters = parser.type_parameters()?;
    parser.expect('(')?;
    let mut parameters = Vec::new();
    while !parser.consume(')') {
        parameters.push(parser.type_ref()?);
    }
    let returns = if parser.consume('V') {
        None
    } else {
        Some(parser.type_ref()?)
    };
    while parser.consume('^') {
        parser.type_ref()?;
    }
    parser.finish()?;
    Ok(MethodSignature {
        type_parameters,
        parameters,
        returns,
    })
}

pub(crate) struct ClassSignature {
    pub(crate) type_parameters: Vec<TypeParameter>,
    pub(crate) superclass: TypeRef,
    pub(crate) interfaces: Vec<TypeRef>,
}

pub(crate) fn class_signature(text: &str) -> Result<ClassSignature, String> {
    let mut parser = Parser::new(text);
    let type_parameters = parser.type_parameters()?;
    let superclass = parser.type_ref()?;
    let mut interfaces = Vec::new();
    while parser.peek().is_some() {
        interfaces.push(parser.type_ref()?);
    }
    Ok(ClassSignature {
        type_parameters,
        superclass,
        interfaces,
    })
}

struct Parser<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, position: 0 }
    }

    fn peek(&self) -> Option<char> {
        self.text[self.position..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let next = self.peek()?;
        self.position += next.len_utf8();
        Some(next)
    }

    fn consume(&mut self, expected: char) -> bool {
        let found = self.peek() == Some(expected);
        if found {
            self.position += expected.len_utf8();
        }
        found
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.consume(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("expected '{expected}'")))
        }
    }

    fn finish(&self) -> Result<(), String> {
        if self.position == self.text.len() {
            Ok(())
        } else {
            Err(self.error("unexpected trailing text"))
        }
    }

    fn error(&self, problem: &str) -> String {
        format!(
            "invalid signature '{}': {problem} at {}",
            self.text, self.position
        )
    }

    fn identifier(&mut self, delimiters: &[char]) -> Result<&'a str, String> {
        let rest = &self.text[self.position..];
        match rest.find(delimiters) {
            Some(length) if length > 0 => {
                self.position += length;
                Ok(&rest[..length])
            }
            _ => Err(self.error("expected a name")),
        }
    }

    fn type_ref(&mut self) -> Result<TypeRef, String> {
        match self.bump() {
            Some('B') => Ok(TypeRef::primitive("byte")),
            Some('C') => Ok(TypeRef::primitive("char")),
            Some('D') => Ok(TypeRef::primitive("double")),
            Some('F') => Ok(TypeRef::primitive("float")),
            Some('I') => Ok(TypeRef::primitive("int")),
            Some('J') => Ok(TypeRef::primitive("long")),
            Some('S') => Ok(TypeRef::primitive("short")),
            Some('Z') => Ok(TypeRef::primitive("boolean")),
            Some('[') => Ok(TypeRef::array(self.type_ref()?)),
            Some('T') => {
                let name = self.identifier(&[';'])?;
                self.expect(';')?;
                Ok(TypeRef::variable(name))
            }
            Some('L') => self.class_type(),
            _ => Err(self.error("expected a type")),
        }
    }

    fn class_type(&mut self) -> Result<TypeRef, String> {
        let mut name = self.identifier(&['<', ';', '.'])?.replace('/', ".");
        let mut arguments = self.arguments()?;
        while self.consume('.') {
            name.push('$');
            name.push_str(self.identifier(&['<', ';', '.'])?);
            arguments = self.arguments()?;
        }
        self.expect(';')?;
        Ok(TypeRef::generic(&name, arguments))
    }

    fn arguments(&mut self) -> Result<Vec<TypeRef>, String> {
        let mut arguments = Vec::new();
        if self.consume('<') {
            while !self.consume('>') {
                arguments.push(self.argument()?);
            }
        }
        Ok(arguments)
    }

    fn argument(&mut self) -> Result<TypeRef, String> {
        if self.consume('*') {
            return Ok(TypeRef::class("java.lang.Object"));
        }
        if !self.consume('+') {
            self.consume('-');
        }
        self.type_ref()
    }

    fn type_parameters(&mut self) -> Result<Vec<TypeParameter>, String> {
        let mut parameters = Vec::new();
        if self.consume('<') {
            while !self.consume('>') {
                parameters.push(self.type_parameter()?);
            }
        }
        Ok(parameters)
    }

    fn type_parameter(&mut self) -> Result<TypeParameter, String> {
        let name = self.identifier(&[':'])?.to_owned();
        let mut bounds = Vec::new();
        while self.consume(':') {
            if self.peek() != Some(':') {
                bounds.push(self.type_ref()?);
            }
        }
        Ok(TypeParameter { name, bounds })
    }
}
