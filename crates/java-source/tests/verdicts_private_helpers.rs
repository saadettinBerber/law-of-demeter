mod judging;

use judging::verdicts_in_c;
use law_of_demeter_core::domain::classification::Types;
use law_of_demeter_core::domain::method::{Reason, Verdict};
use law_of_demeter_core::domain::type_shape::{MethodShape, TypeKind, TypeRef, TypeShape};

const MAP_IMPORT: &str = "import java.util.Map;";

const CART_FIELDS: &str = "Map<String, Cart> carts; Session session;";

fn class(name: &str) -> TypeShape {
    TypeShape::new(name, TypeKind::Class)
}

fn finding_a_cart(name: &str) -> MethodShape {
    MethodShape::new(name)
        .with_parameters(vec![TypeRef::class("java.lang.String")])
        .returning(TypeRef::class("Cart"))
}

fn shop_finding_carts_with(helpers: Vec<MethodShape>) -> Types {
    let shop = helpers.into_iter().fold(class("C"), TypeShape::with_method);
    Types::new()
        .with_shape(shop)
        .with_shape(
            TypeShape::new("java.util.Map", TypeKind::Interface)
                .with_type_parameter("K")
                .with_type_parameter("V")
                .with_method(
                    MethodShape::new("get")
                        .with_parameters(vec![TypeRef::class("java.lang.Object")])
                        .returning(TypeRef::variable("V")),
                ),
        )
        .with_shape(
            class("Session").with_method(
                MethodShape::new("getCart")
                    .reading("cart")
                    .returning(TypeRef::class("Cart")),
            ),
        )
        .with_shape(class("Cart").with_method(MethodShape::new("checkout")))
}

fn checking_out_through(helpers: &str) -> String {
    format!("void checkOut(String id) {{\n    findCart(id).checkout();\n}}\n{helpers}")
}

fn verdicts_of_check_out(helpers: &str, types: &Types) -> Vec<Verdict> {
    verdicts_in_c(
        MAP_IMPORT,
        CART_FIELDS,
        &checking_out_through(helpers),
        "checkOut",
        types,
    )
}

#[test]
fn an_element_a_private_helper_takes_from_a_container_field_is_a_friend() {
    let types = shop_finding_carts_with(vec![finding_a_cart("findCart").as_private()]);
    let helper = "private Cart findCart(String id) {\n    return carts.get(id);\n}";

    let verdicts = verdicts_of_check_out(helper, &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("findCart", 3, Reason::This),
            Verdict::allowed("checkout", 3, Reason::field("carts")),
        ]
    );
}

#[test]
fn a_private_guard_returning_a_local_taken_from_a_container_field_is_a_friend() {
    let types = shop_finding_carts_with(vec![finding_a_cart("findCart").as_private()]);
    let helper = "private Cart findCart(String id) {\n    Cart cart = carts.get(id);\n    if (cart == null) {\n        throw new IllegalStateException(\"no cart \" + id);\n    }\n    return cart;\n}";

    let verdicts = verdicts_of_check_out(helper, &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("findCart", 3, Reason::This),
            Verdict::allowed("checkout", 3, Reason::field("carts")),
        ]
    );
}

#[test]
fn a_private_helper_with_one_stranger_return_stays_a_stranger() {
    let types = shop_finding_carts_with(vec![finding_a_cart("findCart").as_private()]);
    let helper = "private Cart findCart(String id) {\n    if (id == null) {\n        return session.getCart();\n    }\n    return carts.get(id);\n}";

    let verdicts = verdicts_of_check_out(helper, &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("findCart", 3, Reason::This),
            Verdict::violation("checkout", 3, Reason::return_of("findCart")),
        ]
    );
}

#[test]
fn a_helper_a_subclass_can_override_stays_a_stranger() {
    let types = shop_finding_carts_with(vec![finding_a_cart("findCart")]);
    let helper = "Cart findCart(String id) {\n    return carts.get(id);\n}";

    let verdicts = verdicts_of_check_out(helper, &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("findCart", 3, Reason::This),
            Verdict::violation("checkout", 3, Reason::return_of("findCart")),
        ]
    );
}

#[test]
fn mutually_recursive_private_helpers_stay_strangers() {
    let types = shop_finding_carts_with(vec![
        finding_a_cart("findCart").as_private(),
        finding_a_cart("lookUp").as_private(),
    ]);
    let helpers = "private Cart findCart(String id) {\n    if (id == null) {\n        return lookUp(id);\n    }\n    return carts.get(id);\n}\nprivate Cart lookUp(String id) {\n    return findCart(id);\n}";

    let verdicts = verdicts_of_check_out(helpers, &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("findCart", 3, Reason::This),
            Verdict::violation("checkout", 3, Reason::return_of("findCart")),
        ]
    );
}

#[test]
fn a_private_helper_returning_its_parameter_keeps_the_stranger_it_was_given() {
    let types = shop_finding_carts_with(vec![
        MethodShape::new("same")
            .with_parameters(vec![TypeRef::class("Cart")])
            .returning(TypeRef::class("Cart"))
            .as_private(),
    ]);
    let snippet = "void checkOut() {\n    same(session.getCart()).checkout();\n}\nprivate Cart same(Cart cart) {\n    return cart;\n}";

    let verdicts = verdicts_in_c(MAP_IMPORT, CART_FIELDS, snippet, "checkOut", &types);

    assert_eq!(
        verdicts,
        [
            Verdict::allowed("getCart", 3, Reason::field("session")),
            Verdict::allowed("same", 3, Reason::This),
            Verdict::violation("checkout", 3, Reason::return_of("same")),
        ]
    );
}
