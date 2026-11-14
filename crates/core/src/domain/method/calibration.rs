use super::{Call, Expression, Lambda, Method, Reason, Verdict};
use crate::domain::classification::{Classification, Types};
use crate::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn interface(name: &str, parameters: &[&str]) -> TypeShape {
    parameters.iter().fold(
        TypeShape::new(name, TypeKind::Interface),
        |shape, parameter| shape.with_type_parameter(parameter),
    )
}

fn returning(name: &str, parameters: usize, returns: TypeRef) -> MethodShape {
    MethodShape::new(name)
        .with_parameters(vec![TypeRef::class("java.lang.Object"); parameters])
        .returning(returns)
}

fn generic(name: &str, arguments: &[TypeRef]) -> TypeRef {
    TypeRef::generic(name, arguments.to_vec())
}

fn variable(name: &str) -> TypeRef {
    TypeRef::variable(name)
}

fn containers() -> Types {
    let stream = |element: TypeRef| generic("java.util.stream.Stream", &[element]);
    Types::new()
        .with_shape(
            interface("java.util.Map", &["K", "V"])
                .with_method(returning("get", 1, variable("V")))
                .with_method(returning(
                    "values",
                    0,
                    generic("java.util.Collection", &[variable("V")]),
                )),
        )
        .with_shape(
            interface("java.util.Collection", &["E"])
                .with_method(returning("stream", 0, stream(variable("E"))))
                .with_method(returning(
                    "iterator",
                    0,
                    generic("java.util.Iterator", &[variable("E")]),
                )),
        )
        .with_shape(
            interface("java.util.List", &["E"])
                .extending(generic("java.util.Collection", &[variable("E")]))
                .with_method(returning("get", 1, variable("E")))
                .with_method(returning("stream", 0, stream(variable("E")))),
        )
        .with_shape(
            interface("java.util.Iterator", &["E"]).with_method(returning(
                "next",
                0,
                variable("E"),
            )),
        )
        .with_shape(
            interface("java.util.Optional", &["T"])
                .with_method(returning("get", 0, variable("T")))
                .with_method(returning("orElseThrow", 0, variable("T"))),
        )
        .with_shape(
            interface("java.util.stream.Stream", &["T"]).with_method(returning(
                "filter",
                1,
                stream(variable("T")),
            )),
        )
        .with_shape(class("Cart").with_method(MethodShape::new("checkout")))
        .with_shape(class("Item").with_method(MethodShape::new("isPaid")))
        .with_shape(class("CartRepository").with_method(returning(
            "findById",
            1,
            generic("java.util.Optional", &[TypeRef::class("Cart")]),
        )))
}

#[test]
fn an_element_taken_from_a_container_field_is_a_friend() {
    let carts = generic(
        "java.util.Map",
        &[TypeRef::class("java.lang.String"), TypeRef::class("Cart")],
    );
    let check_out = Method::new("Shop", "checkOut", 1).call(
        Call::new(Expression::field("carts", carts), "get", 2)
            .with(Expression::parameter(
                "id",
                TypeRef::class("java.lang.String"),
            ))
            .then("checkout", 2),
    );

    let verdicts = check_out.verdicts(&containers());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("get", 2, Reason::field("carts")),
            Verdict::allowed("checkout", 2, Reason::field("carts")),
        ]
    );
}

#[test]
fn the_value_unwrapped_from_a_repository_result_is_a_friend() {
    let ship = |unwrap: &str| {
        Method::new("Shop", "ship", 1).call(
            Call::new(
                Expression::field("carts", TypeRef::class("CartRepository")),
                "findById",
                2,
            )
            .with(Expression::parameter(
                "id",
                TypeRef::class("java.lang.String"),
            ))
            .then(unwrap, 2)
            .then("checkout", 2),
        )
    };

    let verdicts = [
        ship("get").verdicts(&containers()),
        ship("orElseThrow").verdicts(&containers()),
    ];

    let friend = |unwrap: &str| {
        vec![
            Verdict::allowed("findById", 2, Reason::field("carts")),
            Verdict::allowed(unwrap, 2, Reason::query_result("findById")),
            Verdict::allowed("checkout", 2, Reason::query_result("findById")),
        ]
    };
    assert_eq!(verdicts, [friend("get"), friend("orElseThrow")]);
}

#[test]
fn the_parameter_of_a_lambda_given_to_a_stream_of_a_container_field_is_a_friend() {
    let items = generic("java.util.List", &[TypeRef::class("Item")]);
    let paid = Method::new("Shop", "paid", 1).call(
        Call::new(Expression::field("items", items), "stream", 2)
            .then("filter", 2)
            .with(Lambda::new(&["item"]).call(Call::new(Expression::local("item"), "isPaid", 2))),
    );

    let verdicts = paid.verdicts(&containers());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("stream", 2, Reason::field("items")),
            Verdict::allowed(
                "isPaid",
                2,
                Reason::lambda_parameter("item", Reason::field("items"))
            ),
            Verdict::allowed("filter", 2, Reason::field("items")),
        ]
    );
}

#[test]
fn an_element_taken_from_a_returned_container_stays_a_stranger() {
    let lines = generic("java.util.List", &[TypeRef::class("OrderLine")]);
    let types = containers()
        .with_shape(class("Order").with_method(returning("getLines", 0, lines.clone())))
        .with_shape(class("OrderLine").with_method(MethodShape::new("applyDiscount")));
    let discount = Method::new("Shop", "discount", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getLines",
            2,
        )
        .then("get", 2)
        .with(Expression::literal())
        .then("applyDiscount", 2),
    );

    let verdicts = discount.verdicts(&types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getLines", 2, Reason::parameter("order")),
            Verdict::allowed(
                "get",
                2,
                Reason::classified(
                    crate::domain::classification::Classification::Container,
                    lines
                )
            ),
            Verdict::violation("applyDiscount", 2, Reason::return_of("getLines")),
        ]
    );
}

#[test]
fn values_iterators_and_their_next_elements_of_a_container_field_are_friends() {
    let carts = generic(
        "java.util.Map",
        &[TypeRef::class("java.lang.String"), TypeRef::class("Cart")],
    );
    let check_out_all = Method::new("Shop", "checkOutFirst", 1).call(
        Call::new(Expression::field("carts", carts), "values", 2)
            .then("iterator", 2)
            .then("next", 2)
            .then("checkout", 2),
    );

    let verdicts = check_out_all.verdicts(&containers());

    assert_eq!(
        verdicts,
        ["values", "iterator", "next", "checkout"].map(|call| Verdict::allowed(
            call,
            2,
            Reason::field("carts")
        ))
    );
}

const LOCK: &str = "java.util.concurrent.locks.ReentrantReadWriteLock";
const STAMP: &str = "java.util.concurrent.locks.StampedLock";
const WRITE_LOCK: &str = "java.util.concurrent.locks.ReentrantReadWriteLock$WriteLock";

fn with_project(types: Types, classes: &[&str]) -> Types {
    types.with_project_types(classes.iter().map(|name| (*name).to_owned()))
}

fn libraries() -> Types {
    with_project(
        Types::new()
            .with_shape(class(LOCK).with_method(returning(
                "writeLock",
                0,
                TypeRef::class(WRITE_LOCK),
            )))
            .with_shape(class(WRITE_LOCK).with_method(MethodShape::new("lock")))
            .with_shape(class("java.lang.Class").with_method(returning(
                "getName",
                0,
                TypeRef::class("java.lang.String"),
            )))
            .with_shape(
                class("Shop")
                    .with_method(returning("getClass", 0, TypeRef::class("java.lang.Class")))
                    .with_method(returning("getStamp", 0, TypeRef::class(STAMP))),
            )
            .with_shape(class("Wallet").with_method(MethodShape::new("withdraw")))
            .with_shape(class("Customer").with_method(returning(
                "getWallet",
                0,
                TypeRef::class("Wallet"),
            )))
            .with_shape(class("Order").with_method(returning(
                "getCustomer",
                0,
                TypeRef::class("Customer"),
            ))),
        &["Shop", "Order", "Customer", "Wallet"],
    )
}

#[test]
fn a_call_on_a_library_object_returned_by_a_field_is_allowed() {
    let guard = Method::new("Shop", "guard", 1).call(
        Call::new(
            Expression::field("lock", TypeRef::class(LOCK)),
            "writeLock",
            2,
        )
        .then("lock", 2),
    );

    let verdicts = guard.verdicts(&libraries());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("writeLock", 2, Reason::field("lock")),
            Verdict::allowed("lock", 2, Reason::library(TypeRef::class(WRITE_LOCK))),
        ]
    );
}

#[test]
fn the_name_read_from_the_class_of_this_is_allowed_as_a_value() {
    let describe = Method::new("Shop", "describe", 1)
        .call(Call::new(Expression::This, "getClass", 2).then("getName", 2));

    let verdicts = describe.verdicts(&libraries());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getClass", 2, Reason::This),
            Verdict::allowed(
                "getName",
                2,
                Reason::classified(Classification::Value, TypeRef::class("java.lang.Class"))
            ),
        ]
    );
}

#[test]
fn a_call_on_a_project_object_returned_by_a_parameter_stays_a_violation() {
    let charge = Method::new("Shop", "charge", 1).call(
        Call::new(
            Expression::parameter("order", TypeRef::class("Order")),
            "getCustomer",
            2,
        )
        .then("getWallet", 2),
    );

    let verdicts = charge.verdicts(&libraries());

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCustomer", 2, Reason::parameter("order")),
            Verdict::violation("getWallet", 2, Reason::return_of("getCustomer")),
        ]
    );
}

#[test]
fn a_call_on_a_type_missing_from_the_catalog_stays_unresolved() {
    let guard = Method::new("Shop", "guard", 1)
        .call(Call::new(Expression::This, "getStamp", 2).then("validate", 2));

    let verdicts = guard.verdicts(&libraries());

    assert_eq!(
        verdicts[1],
        Verdict::violation(
            "validate",
            2,
            Reason::unresolved_type(Some(TypeRef::class(STAMP)), Reason::return_of("getStamp"))
        )
    );
}

#[test]
fn library_knowledge_stays_off_when_the_catalog_knows_no_project_types() {
    let types = Types::new()
        .with_shape(class(LOCK).with_method(returning("writeLock", 0, TypeRef::class(WRITE_LOCK))))
        .with_shape(class(WRITE_LOCK).with_method(MethodShape::new("lock")))
        .with_shape(class("Shop"));
    let guard = Method::new("Shop", "guard", 1).call(
        Call::new(
            Expression::field("lock", TypeRef::class(LOCK)),
            "writeLock",
            2,
        )
        .then("lock", 2),
    );

    let verdicts = guard.verdicts(&types);

    assert_eq!(
        verdicts[1],
        Verdict::violation("lock", 2, Reason::return_of("writeLock"))
    );
}
