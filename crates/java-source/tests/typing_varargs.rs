mod judging;

use judging::verdicts_in_c;
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const STREAM: &str = "java.util.stream.Stream";
const OPTIONAL: &str = "java.util.Optional";
const LIST: &str = "java.util.List";
const FUNCTION: &str = "java.util.function.Function";
const CONSUMER: &str = "java.util.function.Consumer";
const STRING: &str = "java.lang.String";
const OBJECT: &str = "java.lang.Object";

const STREAM_IMPORT: &str = "import java.util.stream.Stream;";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn interface(name: &str, parameters: &[&str]) -> TypeShape {
    parameters.iter().fold(
        TypeShape::new(name, TypeKind::Interface),
        |shape, parameter| shape.with_type_parameter(parameter),
    )
}

fn generic(name: &str, arguments: &[TypeRef]) -> TypeRef {
    TypeRef::generic(name, arguments.to_vec())
}

fn variable(name: &str) -> TypeRef {
    TypeRef::variable(name)
}

fn order() -> TypeRef {
    TypeRef::class("Order")
}

fn stream_of(element: TypeRef) -> TypeRef {
    generic(STREAM, &[element])
}

fn streams(varargs_returns: TypeRef) -> Types {
    let stream = interface(STREAM, &["T"])
        .with_method(
            MethodShape::new("of")
                .as_static()
                .with_type_parameter("T")
                .with_parameters(vec![variable("T")])
                .returning(stream_of(variable("T"))),
        )
        .with_method(
            MethodShape::new("of")
                .as_static()
                .as_varargs()
                .with_type_parameter("T")
                .with_parameters(vec![TypeRef::array(variable("T"))])
                .returning(varargs_returns),
        )
        .with_method(
            MethodShape::new("flatMap")
                .with_type_parameter("R")
                .with_parameters(vec![generic(
                    FUNCTION,
                    &[variable("T"), stream_of(variable("R"))],
                )])
                .returning(stream_of(variable("R"))),
        )
        .with_method(MethodShape::new("toList").returning(generic(LIST, &[variable("T")])))
        .with_method(
            MethodShape::new("forEach").with_parameters(vec![generic(CONSUMER, &[variable("T")])]),
        );
    Types::new()
        .with_shape(stream)
        .with_shape(
            interface(FUNCTION, &["T", "R"]).with_method(
                MethodShape::new("apply")
                    .with_parameters(vec![variable("T")])
                    .returning(variable("R"))
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(CONSUMER, &["T"]).with_method(
                MethodShape::new("accept")
                    .with_parameters(vec![variable("T")])
                    .as_abstract(),
            ),
        )
        .with_shape(
            interface(OPTIONAL, &["T"])
                .with_method(MethodShape::new("stream").returning(stream_of(variable("T")))),
        )
        .with_shape(
            interface(LIST, &["E"]).with_method(
                MethodShape::new("get")
                    .with_parameters(vec![TypeRef::primitive("int")])
                    .returning(variable("E")),
            ),
        )
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer"))),
        )
        .with_shape(class("Customer").with_method(MethodShape::new("rename")))
}

fn varargs_method(name: &str, parameters: Vec<TypeRef>, returns: &str) -> MethodShape {
    MethodShape::new(name)
        .as_varargs()
        .with_parameters(parameters)
        .returning(TypeRef::class(returns))
}

fn array_of(name: &str) -> TypeRef {
    TypeRef::array(TypeRef::class(name))
}

fn helpers() -> Types {
    streams(stream_of(variable("T")))
        .with_shape(
            class(STRING)
                .with_method(
                    varargs_method(
                        "format",
                        vec![TypeRef::class(STRING), array_of(OBJECT)],
                        STRING,
                    )
                    .as_static(),
                )
                .with_method(MethodShape::new("trim").returning(TypeRef::class(STRING)))
                .with_method(MethodShape::new("isEmpty").returning(TypeRef::primitive("boolean"))),
        )
        .with_shape(
            class("Picker")
                .with_method(varargs_method("pick", vec![array_of("Order")], "Order").as_static()),
        )
        .with_shape(
            class("Actions").with_method(
                MethodShape::new("run")
                    .as_static()
                    .as_varargs()
                    .with_parameters(vec![TypeRef::array(generic(CONSUMER, &[order()]))]),
            ),
        )
        .with_shape(class("Builder").with_method(varargs_method(
            "add",
            vec![array_of(STRING)],
            "Builder",
        )))
        .with_shape(
            class("Order")
                .with_method(MethodShape::new("getCustomer").returning(TypeRef::class("Customer")))
                .with_method(MethodShape::new("getBuilder").returning(TypeRef::class("Builder"))),
        )
        .with_shape(
            class("Printer")
                .with_method(varargs_method("print", vec![array_of(OBJECT)], "Wallet"))
                .with_method(varargs_method(
                    "print",
                    vec![TypeRef::class(STRING), array_of(OBJECT)],
                    "Customer",
                )),
        )
        .with_shape(
            class("Joiner")
                .with_method(varargs_method("join", vec![array_of(STRING)], "Wallet"))
                .with_method(varargs_method(
                    "join",
                    vec![TypeRef::array(TypeRef::primitive("int"))],
                    "Customer",
                )),
        )
        .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
        .with_shape(class("Customer").with_method(MethodShape::new("rename")))
}

#[test]
fn a_variable_arity_method_matches_without_any_argument_for_its_variable_parameter() {
    let snippet = "void ship() {\n    Picker.pick().getCustomer().rename();\n}";

    let verdicts = verdicts_in_c("", "", snippet, "ship", &helpers());

    assert_eq!(
        verdicts[2],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}

#[test]
fn a_lambda_at_a_variable_arity_position_takes_the_element_type_of_the_array_parameter() {
    let snippet = "void ship() {\n    Actions.run(first -> { first.getCustomer().rename(); }, second -> { second.getCustomer().rename(); });\n}";

    let verdicts = verdicts_in_c("", "", snippet, "ship", &helpers());

    assert_eq!(
        [&verdicts[1], &verdicts[3]],
        [
            &Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
            &Verdict::violation("rename", 3, Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn a_variable_arity_builder_method_stays_fluent_with_any_number_of_arguments() {
    let snippet = "void build(Order order, String first, String second, String third) {\n    order.getBuilder().add(first, second).add(third);\n}";

    let verdicts = verdicts_in_c("", "", snippet, "build", &helpers());

    let builder = TypeRef::class("Builder");
    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getBuilder", 3, Reason::parameter("order")),
            Verdict::allowed("add", 3, Reason::fluent(builder.clone())),
            Verdict::allowed("add", 3, Reason::fluent(builder)),
        ]
    );
}

#[test]
fn the_most_specific_of_several_fitting_variable_arity_overloads_decides_the_return() {
    let snippet = "void print(String title, Order first, Order second) {\n    printer.print(title, first, second).withdraw();\n}";

    let verdicts = verdicts_in_c("", "Printer printer;", snippet, "print", &helpers());

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 3, Reason::return_of("print"))
    );
}

#[test]
fn the_variable_arity_overload_whose_element_type_fits_the_arguments_decides_the_return() {
    let snippet =
        "void join(String first, String second) {\n    joiner.join(first, second).withdraw();\n}";

    let verdicts = verdicts_in_c("", "Joiner joiner;", snippet, "join", &helpers());

    assert_eq!(
        verdicts[1],
        Verdict::violation("withdraw", 3, Reason::return_of("join"))
    );
}

#[test]
fn one_argument_still_picks_the_fixed_arity_overload() {
    let snippet = "void ship(Order order) {\n    Stream.of(order).forEach(each -> { each.getCustomer().rename(); });\n}";

    let verdicts = verdicts_in_c(
        STREAM_IMPORT,
        "",
        snippet,
        "ship",
        &streams(stream_of(TypeRef::class("Wallet"))),
    );

    assert_eq!(
        verdicts[2],
        Verdict::violation("rename", 3, Reason::return_of("getCustomer"))
    );
}
