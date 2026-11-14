use law_of_demeter_core::domain::method::Reason;

pub fn describe(reason: &Reason) -> String {
    match reason {
        Reason::ReturnOf(call) => {
            format!("{call}() çağrısının döndürdüğü nesne f'nin arkadaşı değil")
        }
        _ => String::new(),
    }
}
