mod support;

use law_of_demeter_java_source::{Expression, LiteralType, LocalVariable, Statement};
use support::body_of;

fn literal_type_of(literal: &str) -> LiteralType {
    let body = body_of(&format!("void f() {{\n var x = {literal};\n}}"));
    match &body[0] {
        Statement::Local(LocalVariable {
            initializer: Some(Expression::Literal(literal_type)),
            ..
        }) => *literal_type,
        other => panic!("expected a literal initializer, got {other:?}"),
    }
}

#[test]
fn an_integer_literal_is_an_int() {
    assert_eq!(literal_type_of("1"), LiteralType::Int);
}

#[test]
fn hexadecimal_and_binary_literals_are_ints() {
    assert_eq!(literal_type_of("0xFF"), LiteralType::Int);
    assert_eq!(literal_type_of("0b101"), LiteralType::Int);
}

#[test]
fn an_integer_literal_with_an_l_suffix_is_a_long() {
    assert_eq!(literal_type_of("1L"), LiteralType::Long);
}

#[test]
fn a_decimal_literal_is_a_double() {
    assert_eq!(literal_type_of("1.5"), LiteralType::Double);
}

#[test]
fn a_decimal_literal_with_a_d_suffix_is_a_double() {
    assert_eq!(literal_type_of("1d"), LiteralType::Double);
}

#[test]
fn a_decimal_literal_with_an_f_suffix_is_a_float() {
    assert_eq!(literal_type_of("1.5f"), LiteralType::Float);
}

#[test]
fn true_and_false_are_booleans() {
    assert_eq!(literal_type_of("true"), LiteralType::Boolean);
    assert_eq!(literal_type_of("false"), LiteralType::Boolean);
}

#[test]
fn a_character_literal_is_a_char() {
    assert_eq!(literal_type_of("'a'"), LiteralType::Char);
}

#[test]
fn a_string_literal_is_a_string() {
    assert_eq!(literal_type_of("\"text\""), LiteralType::String);
}

#[test]
fn a_text_block_is_a_string() {
    assert_eq!(literal_type_of("\"\"\"\n hi\n \"\"\""), LiteralType::String);
}

#[test]
fn null_is_the_null_type() {
    assert_eq!(literal_type_of("null"), LiteralType::Null);
}
