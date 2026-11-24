#![allow(non_camel_case_types)]

pub struct r#type;

impl r#type {
    pub fn touch(&self) {}

    pub fn r#match(&self, other: &r#type) {
        other.touch();
    }
}
