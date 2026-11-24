use cafebabe::attributes::{AttributeData, CodeData};
use cafebabe::bytecode::Opcode;
use cafebabe::descriptors::{FieldType, ReturnDescriptor};
use cafebabe::{MethodAccessFlags, MethodInfo};

pub(crate) fn returns_receiver(method: &MethodInfo) -> bool {
    if method.access_flags.contains(MethodAccessFlags::STATIC) {
        return false;
    }
    let Some(code) = code_of(method) else {
        return false;
    };
    let Some(bytecode) = &code.bytecode else {
        return false;
    };
    let opcodes = &bytecode.opcodes;
    if opcodes
        .iter()
        .any(|(_, opcode)| overwrites_receiver(opcode))
    {
        return false;
    }
    let returns: Vec<_> = opcodes
        .iter()
        .enumerate()
        .filter(|(_, (_, opcode))| matches!(opcode, Opcode::Areturn))
        .collect();
    !returns.is_empty()
        && returns.iter().all(|(index, (offset, _))| {
            index
                .checked_sub(1)
                .and_then(|previous| opcodes.get(previous))
                .is_some_and(|(_, opcode)| matches!(opcode, Opcode::Aload(0)))
                && !code
                    .exception_table
                    .iter()
                    .any(|entry| usize::from(entry.handler_pc) == *offset)
                && !opcodes
                    .iter()
                    .any(|(source, opcode)| jumps_to(*source, opcode, *offset))
        })
}

pub(crate) fn returns_exact_type(method: &MethodInfo) -> bool {
    let Some(code) = code_of(method) else {
        return false;
    };
    let Some(bytecode) = &code.bytecode else {
        return false;
    };
    let [
        (.., Opcode::New(created)),
        (.., Opcode::Dup),
        (.., Opcode::Invokespecial(constructor)),
        (.., Opcode::Areturn),
    ] = bytecode.opcodes.as_slice()
    else {
        return false;
    };
    code.exception_table.is_empty()
        && constructor.class_name == *created
        && constructor.name_and_type.name == "<init>"
        && constructor.name_and_type.descriptor == "()V"
        && matches!(&method.descriptor.return_type, ReturnDescriptor::Return(returned) if matches!(&returned.field_type, FieldType::Object(name) if **name == **created))
}

fn code_of<'m, 'a>(method: &'m MethodInfo<'a>) -> Option<&'m CodeData<'a>> {
    method
        .attributes
        .iter()
        .find_map(|attribute| match &attribute.data {
            AttributeData::Code(code) => Some(code),
            _ => None,
        })
}

fn overwrites_receiver(opcode: &Opcode) -> bool {
    matches!(
        opcode,
        Opcode::Astore(0)
            | Opcode::Istore(0)
            | Opcode::Lstore(0)
            | Opcode::Fstore(0)
            | Opcode::Dstore(0)
            | Opcode::Iinc(0, _)
            | Opcode::Jsr(_)
            | Opcode::Ret(_)
    )
}

fn jumps_to(source: usize, opcode: &Opcode, target: usize) -> bool {
    let reaches = |jump: i32| source.checked_add_signed(jump as isize) == Some(target);
    match opcode {
        Opcode::Goto(jump)
        | Opcode::IfAcmpeq(jump)
        | Opcode::IfAcmpne(jump)
        | Opcode::IfIcmpeq(jump)
        | Opcode::IfIcmpge(jump)
        | Opcode::IfIcmpgt(jump)
        | Opcode::IfIcmple(jump)
        | Opcode::IfIcmplt(jump)
        | Opcode::IfIcmpne(jump)
        | Opcode::Ifeq(jump)
        | Opcode::Ifge(jump)
        | Opcode::Ifgt(jump)
        | Opcode::Ifle(jump)
        | Opcode::Iflt(jump)
        | Opcode::Ifne(jump)
        | Opcode::Ifnonnull(jump)
        | Opcode::Ifnull(jump) => reaches(*jump),
        Opcode::Lookupswitch(table) => {
            reaches(table.default) || table.match_offsets.iter().any(|(_, jump)| reaches(*jump))
        }
        Opcode::Tableswitch(table) => {
            reaches(table.default) || table.jumps.iter().any(|jump| reaches(*jump))
        }
        _ => false,
    }
}
