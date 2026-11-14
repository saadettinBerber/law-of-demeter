use cafebabe::attributes::AttributeData;
use cafebabe::bytecode::Opcode;
use cafebabe::constant_pool::MemberRef;
use cafebabe::{ClassFile, FieldAccessFlags, MethodInfo};

pub(crate) enum Access {
    Read(String),
    Write(String),
}

pub(crate) fn of(class: &ClassFile, method: &MethodInfo) -> Option<Access> {
    let opcodes = opcodes_of(method)?;
    match (opcodes.as_slice(), method.descriptor.parameters.len()) {
        ([Opcode::Aload(0), Opcode::Getfield(field), returned], 0)
            if returns_value(returned) && is_own_instance_field(class, field) =>
        {
            Some(Access::Read(field.name_and_type.name.to_string()))
        }
        (
            [
                Opcode::Aload(0),
                loaded,
                Opcode::Putfield(field),
                Opcode::Return,
            ],
            1,
        ) if loads_first_parameter(loaded) && is_own_instance_field(class, field) => {
            Some(Access::Write(field.name_and_type.name.to_string()))
        }
        _ => None,
    }
}

fn opcodes_of<'m, 'a>(method: &'m MethodInfo<'a>) -> Option<Vec<&'m Opcode<'a>>> {
    let bytecode = method
        .attributes
        .iter()
        .find_map(|attribute| match &attribute.data {
            AttributeData::Code(code) => code.bytecode.as_ref(),
            _ => None,
        })?;
    Some(bytecode.opcodes.iter().map(|(_, opcode)| opcode).collect())
}

fn returns_value(opcode: &Opcode) -> bool {
    matches!(
        opcode,
        Opcode::Ireturn | Opcode::Lreturn | Opcode::Freturn | Opcode::Dreturn | Opcode::Areturn
    )
}

fn loads_first_parameter(opcode: &Opcode) -> bool {
    matches!(
        opcode,
        Opcode::Iload(1)
            | Opcode::Lload(1)
            | Opcode::Fload(1)
            | Opcode::Dload(1)
            | Opcode::Aload(1)
    )
}

fn is_own_instance_field(class: &ClassFile, field: &MemberRef) -> bool {
    class.fields.iter().any(|own| {
        own.name == field.name_and_type.name
            && !own
                .access_flags
                .intersects(FieldAccessFlags::STATIC | FieldAccessFlags::SYNTHETIC)
    })
}
