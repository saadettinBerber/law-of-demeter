use crate::Statement;

use super::MethodMapper;

impl MethodMapper<'_> {
    pub(super) fn statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Expression(expression) | Statement::Return(Some(expression)) => {
                let value = self.expression(expression);
                self.steps.evaluate(value);
            }
            _ => {}
        }
    }
}
