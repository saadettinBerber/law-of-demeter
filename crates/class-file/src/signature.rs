use law_of_demeter_core::domain::type_shape::TypeRef;

pub(crate) fn field_type(text: &str) -> Result<TypeRef, String> {
    let mut parser = Parser::new(text);
    let type_ref = parser.type_ref()?;
    parser.finish()?;
    Ok(type_ref)
}

pub(crate) fn class_supertypes(text: &str) -> Result<Vec<TypeRef>, String> {
    let mut parser = Parser::new(text);
    parser.skip_type_parameters()?;
    let mut supertypes = Vec::new();
    while parser.peek().is_some() {
        supertypes.push(parser.type_ref()?);
    }
    Ok(supertypes)
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

    fn skip_type_parameters(&mut self) -> Result<(), String> {
        if self.consume('<') {
            while !self.consume('>') {
                self.identifier(&[':'])?;
                while self.consume(':') {
                    if self.peek() != Some(':') {
                        self.type_ref()?;
                    }
                }
            }
        }
        Ok(())
    }
}
