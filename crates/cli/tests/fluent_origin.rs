use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

const SOURCE: &str = "package shop;

public class OrderService {
    void onStranger(Order order) {
        order.getBuilder().add(1);
        order.getBuilder().isEmpty();
    }
    void onSame(Builder builder) {
        builder.same().finish();
    }
    void onOther(Builder builder) {
        builder.other().finish();
    }
}

class Order {
    private Builder builder;
    Builder getBuilder() { return builder; }
}

final class Builder {
    private int count;
    private Builder next;
    Builder add(int amount) { count += amount; return this; }
    boolean isEmpty() { return count == 0; }
    Builder same() { return this; }
    Builder other() { return next; }
    void finish() { count++; }
}
";

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_fluent_call_on_a_stranger_is_reported_like_an_observer() {
    let project = CompiledProject::new(SOURCE);

    let output = project.check("onStranger");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":5  shop.OrderService.onStranger  add()  Violation: the object returned by getBuilder() is not a friend of f",
            ":6  shop.OrderService.onStranger  isEmpty()  Violation: the object returned by getBuilder() is not a friend of f",
            ":5  shop.OrderService.onStranger  getBuilder()  Allowed: order is an argument of f",
            ":6  shop.OrderService.onStranger  getBuilder()  Allowed: order is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_call_returning_another_instance_of_the_same_type_does_not_keep_friendship() {
    let project = CompiledProject::new(SOURCE);

    let output = project.check("onOther");

    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        project.verdicts(&output),
        [
            ":12  shop.OrderService.onOther  finish()  Violation: the object returned by other() is not a friend of f",
            ":12  shop.OrderService.onOther  other()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_call_proven_to_return_its_receiver_keeps_friendship() {
    let project = CompiledProject::new(SOURCE);

    let output = project.check("onSame");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":9  shop.OrderService.onSame  same()  Allowed: builder is an argument of f",
            ":9  shop.OrderService.onSame  finish()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_overridable_receiver_return_does_not_prove_runtime_identity() {
    let project = CompiledProject::scenario(
        "builder.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }
         class OtherBuilder extends Builder { Builder next; @Override Builder same() { return next; } }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_final_receiver_return_keeps_friendship_on_an_open_class() {
    let project = CompiledProject::scenario(
        "builder.same().finish();",
        "class Builder { final Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
            ":4  shop.OrderService.inspect  finish()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_branch_returning_another_instance_prevents_identity_evidence() {
    let project = CompiledProject::scenario(
        "builder.choose(true).finish();",
        "final class Builder { Builder next; Builder choose(boolean other) { return other ? next : this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by choose() is not a friend of f",
            ":4  shop.OrderService.inspect  choose()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn the_selected_overload_supplies_receiver_identity_evidence() {
    let project = CompiledProject::scenario(
        "builder.same(1).finish(); builder.same(\"next\").finish();",
        "final class Builder { Builder next; Builder same(int amount) { return this; } Builder same(String name) { return next; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
            ":4  shop.OrderService.inspect  finish()  Allowed: builder is an argument of f",
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_receiver_returning_a_supertype_still_keeps_the_same_friend() {
    let project = CompiledProject::scenario(
        "builder.same().finish();",
        "class Base { void finish() {} } final class Builder extends Base { Base same() { return this; } }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
            ":4  shop.OrderService.inspect  finish()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_factory_allocation_proves_the_target_of_an_open_builder_chain() {
    let project = CompiledProject::scenario(
        "Builder.fresh().same().same().finish();",
        "class Builder { static Builder fresh() { return new Builder(); } Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  fresh()  Allowed: static call on type shop.Builder",
            ":4  shop.OrderService.inspect  same()  Allowed: the object returned by static call fresh() is a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: the object returned by static call fresh() is a friend of f",
            ":4  shop.OrderService.inspect  finish()  Allowed: the object returned by static call fresh() is a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_receiver_returning_method_reference_preserves_its_input_friendship() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Builder::same).forEach(Builder::finish);",
        "final class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    let verdicts = project.verdicts(&output);
    let finish: Vec<_> = verdicts
        .iter()
        .filter(|line| line.contains("  finish()  "))
        .collect();
    assert_eq!(
        finish,
        [
            ":4  shop.OrderService.inspect  finish()  Allowed: method reference Builder::finish: the object returned by static call of() is a friend of f"
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_method_reference_returning_another_instance_loses_input_friendship() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Builder::other).forEach(Builder::finish);",
        "final class Builder { Builder next; Builder other() { return next; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    let finish: Vec<_> = verdicts
        .iter()
        .filter(|line| line.contains("  finish()  "))
        .collect();
    assert_eq!(
        finish,
        [
            ":4  shop.OrderService.inspect  finish()  Violation: method reference Builder::finish: the object returned by other() is not a friend of f"
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac and cached Lombok 1.18.30; run with --ignored"]
fn a_lombok_builder_chain_keeps_its_concrete_receiver() {
    let lombok = PathBuf::from(std::env::var_os("HOME").expect("home directory"))
        .join(".m2/repository/org/projectlombok/lombok/1.18.30/lombok-1.18.30.jar");
    assert!(
        lombok.is_file(),
        "cached Lombok is required: {}",
        lombok.display()
    );
    let project = CompiledProject::compile(
        "package shop;
         public class OrderService {
             void inspect() {
                 Order.builder().id(1).total(2).build();
             }
         }
         @lombok.Builder class Order { private int id; private int total; }",
        Some(lombok),
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  builder()  Allowed: static call on type shop.Order",
            ":4  shop.OrderService.inspect  id()  Allowed: the object returned by static call builder() is a friend of f",
            ":4  shop.OrderService.inspect  total()  Allowed: the object returned by static call builder() is a friend of f",
            ":4  shop.OrderService.inspect  build()  Allowed: the object returned by static call builder() is a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_variable_arity_receiver_return_keeps_friendship() {
    let project = CompiledProject::scenario(
        "builder.add(\"first\", \"second\").add(\"third\").finish();",
        "final class Builder { Builder add(String... names) { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  add()  Allowed: builder is an argument of f",
            ":4  shop.OrderService.inspect  add()  Allowed: builder is an argument of f",
            ":4  shop.OrderService.inspect  finish()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn widening_the_return_type_does_not_select_the_base_body_for_a_runtime_override() {
    let project = CompiledProject::scenario(
        "builder.same().again().finish();",
        "class Base { Base again() { return this; } void finish() {} } final class Builder extends Base { Builder next; Base same() { return this; } @Override Base again() { return next; } }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by again() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
            ":4  shop.OrderService.inspect  again()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn splitting_a_known_allocation_into_a_local_preserves_receiver_identity() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn reassignment_to_an_unknown_runtime_type_removes_local_allocation_evidence() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); local = builder; local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn allocating_a_replacement_argument_does_not_prove_the_already_evaluated_receiver() {
    let project = CompiledProject::scenario(
        "Builder local = builder; local.same(local = new Builder()).finish();",
        "class Builder { Builder same(Builder replacement) { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_new_binding_does_not_inherit_an_earlier_scopes_allocation_evidence() {
    let project = CompiledProject::scenario(
        "{ Builder local = new Builder(); local.same().finish(); } { Builder local = builder; local.same().finish(); }",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn allocation_evidence_must_hold_on_both_branches() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); if (builder != null) { local = builder; } else { local = new Builder(); } local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_var_alias_of_a_known_allocation_keeps_receiver_identity() {
    let project = CompiledProject::scenario(
        "var local = new Builder(); Builder alias = local; alias.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: local variable alias: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  finish()  Allowed: local variable alias: local variable local: object created in f (shop.Builder)",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_argument_reassignment_does_not_change_a_stranger_receiver_already_evaluated() {
    let project = CompiledProject::scenario(
        "Builder local = builder.other(); local.same(local = new Builder()).finish();",
        "class Builder { Builder next; Builder other() { return next; } Builder same(Builder replacement) { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Violation: local variable local: the object returned by other() is not a friend of f",
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  other()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_anonymous_subclass_allocation_does_not_prove_the_base_method_body() {
    let project = CompiledProject::scenario(
        "new Builder() { @Override Builder same() { return builder; } }.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_later_argument_assignment_does_not_change_an_earlier_argument_receiver() {
    let project = CompiledProject::scenario(
        "Builder local = builder.other(); builder.take(local.count(), local = new Builder());",
        "class Builder { Builder next; Builder other() { return next; } int count() { return 1; } void take(int count, Builder replacement) {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  count()  Violation: local variable local: the object returned by other() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_bound_reference_keeps_its_receiver_before_a_later_argument_assignment() {
    let project = CompiledProject::scenario(
        "Builder local = builder.other(); builder.take(local::finish, local = new Builder());",
        "class Builder { Builder next; Builder other() { return next; } void finish() {} void take(Runnable action, Builder replacement) {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: local variable local: the object returned by other() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn receiver_and_argument_captures_preserve_distinct_values_and_evaluate_calls_once() {
    let project = CompiledProject::scenario(
        "Builder local = builder.other(); local.take(local.count(), local = new Builder());",
        "class Builder { Builder next; Builder other() { return next; } int count() { return 1; } void take(int count, Builder replacement) {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  count()  Violation: local variable local: the object returned by other() is not a friend of f",
            ":4  shop.OrderService.inspect  take()  Violation: local variable local: the object returned by other() is not a friend of f",
            ":4  shop.OrderService.inspect  other()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn constructor_arguments_keep_the_receiver_seen_by_an_earlier_argument() {
    let project = CompiledProject::scenario(
        "Builder local = builder.other(); new Holder(local.count(), local = new Builder());",
        "class Builder { Builder next; Builder other() { return next; } int count() { return 1; } } class Holder { Holder(int count, Builder replacement) {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  count()  Violation: local variable local: the object returned by other() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_conditional_argument_allocation_does_not_prove_an_existing_local() {
    let project = CompiledProject::scenario(
        "Builder local = builder; builder.take(builder != null ? local = new Builder() : null); local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} void take(Builder replacement) {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_unwritten_allocation_keeps_its_identity_across_loops() {
    for loop_body in [
        "for (int index = 0; index < 2; index++) {}",
        "while (builder != null) { break; }",
        "do {} while (builder != null);",
        "for (Builder item : new Builder[0]) {}",
    ] {
        let project = CompiledProject::scenario(
            &format!("Builder local = new Builder(); {loop_body} local.same().finish();"),
            "class Builder { Builder same() { return this; } void finish() {} }",
        );

        let output = project.check("inspect");

        assert_eq!(
            output.status.code(),
            Some(0),
            "{loop_body}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert_eq!(
            project.verdicts(&output),
            [
                ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
                ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
            ]
        );
    }
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_unwritten_allocation_keeps_its_identity_across_exception_handling() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); try { int count = 1; } catch (RuntimeException ignored) {} finally {} local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_unwritten_allocation_keeps_its_identity_inside_nested_loops() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); for (int index = 0; index < 2; index++) { while (builder != null) { local.same().finish(); break; } }",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_repeated_write_invalidates_allocation_evidence_before_the_loop_body() {
    for loop_body in [
        "for (int index = 0; index < 2; index++) { local.same().finish(); local = builder; }",
        "for (int index = 0; index < 2; local = builder, index++) { local.same().finish(); }",
        "do { local.same().finish(); local = builder; } while (builder != null);",
        "for (int index = 0; index < 2; index++) { while (builder != null) { local = builder; break; } local.same().finish(); }",
    ] {
        let project = CompiledProject::scenario(
            &format!("Builder local = new Builder(); {loop_body}"),
            "class Builder { Builder same() { return this; } void finish() {} }",
        );

        let output = project.check("inspect");

        assert_eq!(output.status.code(), Some(1), "{loop_body}");
        let verdicts = project.verdicts(&output);
        assert_eq!(
            verdicts
                .iter()
                .filter(|line| line.contains("  Violation: "))
                .collect::<Vec<_>>(),
            [
                ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ],
            "{loop_body}"
        );
    }
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn writes_in_other_bindings_do_not_invalidate_an_outer_allocation() {
    for loop_body in [
        "while (builder != null) { builder.local = builder; break; }",
        "for (int index = 0; index < 2; index++) { new Runnable() { public void run() { Builder local = builder; local = new Builder(); } }; }",
    ] {
        let project = CompiledProject::scenario(
            &format!("Builder local = new Builder(); {loop_body} local.same().finish();"),
            "class Builder { Builder local; Builder same() { return this; } void finish() {} }",
        );

        let output = project.check("inspect");

        assert_eq!(output.status.code(), Some(0), "{loop_body}");
        assert_eq!(
            project.verdicts(&output),
            [
                ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
                ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
            ]
        );
    }
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_finally_reassignment_preserves_the_receiver_seen_before_that_assignment() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); try { local.same().finish(); } finally { local = builder; } local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_finally_allocation_supplies_identity_for_the_following_call() {
    let project = CompiledProject::scenario(
        "Builder local = builder; try {} finally { local = new Builder(); } local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_catch_cannot_reuse_identity_that_the_try_body_may_have_reassigned() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); try { local = builder; builder.touch(); } catch (RuntimeException ignored) { local.same().finish(); }",
        "class Builder { Builder same() { return this; } void finish() {} void touch() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_unwritten_allocation_keeps_its_identity_across_switch_branches() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); switch (builder == null ? 0 : 1) { case 0: break; default: break; } local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: local variable local: object created in f (shop.Builder)",
            ":4  shop.OrderService.inspect  finish()  Allowed: local variable local: object created in f (shop.Builder)",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_switch_write_prevents_reusing_allocation_evidence_after_the_branch() {
    let project = CompiledProject::scenario(
        "Builder local = new Builder(); switch (builder == null ? 0 : 1) { case 0: local = builder; break; default: break; } local.same().finish();",
        "class Builder { Builder same() { return this; } void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac and javap; run with --ignored"]
fn an_inaccessible_private_overload_cannot_supply_receiver_identity_evidence() {
    let project = CompiledProject::scenario(
        "builder.same(1).finish();",
        "class Base { Base next; private Base same(int count) { return this; } Base same(Object value) { return next; } void finish() {} } class Builder extends Base {}",
    );
    assert!(
        project
            .bytecode()
            .contains("same:(Ljava/lang/Object;)Lshop/Base;")
    );

    let output = project.check("inspect");

    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac and javap; run with --ignored"]
fn an_inaccessible_private_overload_cannot_supply_the_return_type() {
    let project = CompiledProject::scenario(
        "((Base) builder).same(1).finish();",
        "class Base { Base next; private String same(int count) { return \"value\"; } Base same(Object value) { return next; } void finish() {} } class Builder extends Base {}",
    );
    assert!(
        project
            .bytecode()
            .contains("same:(Ljava/lang/Object;)Lshop/Base;")
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_inaccessible_private_overload_cannot_supply_reference_identity_evidence() {
    let project = CompiledProject::scenario(
        "java.util.stream.IntStream.of(1).mapToObj(builder::same).forEach(Base::finish);",
        "class Base { Base next; private Base same(int count) { return this; } Base same(Object value) { return next; } void finish() {} } class Builder extends Base {}",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: method reference Base::finish: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_accessible_private_overload_keeps_its_receiver_on_another_instance() {
    let project = CompiledProject::new(
        "package shop;
public class OrderService {
    void inspect(OrderService other) {
        other.same(1).finish();
    }
    private OrderService same(int count) { return this; }
    void finish() {}
}",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: other is an argument of f",
            ":4  shop.OrderService.inspect  finish()  Allowed: other is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_nestmate_can_call_a_private_overload_through_its_declaring_type() {
    let project = CompiledProject::new("package shop;
public class OrderService {
    void inspect(Builder builder) {
        ((Base) builder).same(1).finish();
    }
    static class Base { Base next; private Base same(int count) { return this; } Base same(Object value) { return next; } void finish() {} }
    static class Builder extends Base {}
}");

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
            ":4  shop.OrderService.inspect  finish()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac and javap; run with --ignored"]
fn a_private_overload_is_not_inherited_even_when_the_caller_is_a_nestmate() {
    let project = CompiledProject::new("package shop;
public class OrderService {
    void inspect(Builder builder) {
        builder.same(1).finish();
    }
    static class Base { Base next; private Base same(int count) { return this; } Base same(Object value) { return next; } void finish() {} }
    static class Builder extends Base {}
}");
    let bytecode = project.bytecode();
    assert!(
        bytecode.contains("same:(Ljava/lang/Object;)Lshop/OrderService$Base;"),
        "{bytecode}"
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        project.verdicts(&output),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: the object returned by same() is not a friend of f",
            ":4  shop.OrderService.inspect  same()  Allowed: builder is an argument of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac and javap; run with --ignored"]
fn an_inaccessible_static_overload_does_not_turn_an_instance_reference_into_a_factory() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Builder::same).forEach(Base::finish);",
        "class Base { Base next; private static Base same(Builder receiver) { return receiver; } Base same() { return next; } void finish() {} } class Builder extends Base {}",
    );
    assert!(project.bytecode().contains("same:()Lshop/Base;"));

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: method reference Base::finish: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_base_qualifier_preserves_the_final_functional_receivers_identity() {
    for (body, expected) in [
        (
            "builder.same().finish();",
            ":4  shop.OrderService.inspect  finish()  Allowed: builder is an argument of f",
        ),
        (
            "java.util.List.of(builder).stream().map(value -> value.same()).forEach(Base::finish);",
            ":4  shop.OrderService.inspect  finish()  Allowed: method reference Base::finish: lambda parameter value: the object returned by static call of() is a friend of f",
        ),
        (
            "java.util.List.of(builder).stream().map(Builder::same).forEach(Base::finish);",
            ":4  shop.OrderService.inspect  finish()  Allowed: method reference Base::finish: the object returned by static call of() is a friend of f",
        ),
        (
            "java.util.List.of(builder).stream().map(Base::same).forEach(Base::finish);",
            ":4  shop.OrderService.inspect  finish()  Allowed: method reference Base::finish: the object returned by static call of() is a friend of f",
        ),
    ] {
        let project = CompiledProject::scenario(
            body,
            "class Base { Base same() { return this; } void finish() {} } final class Builder extends Base {}",
        );

        let output = project.check("inspect");

        assert_eq!(
            output.status.code(),
            Some(0),
            "{body}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        let verdicts = project.verdicts(&output);
        assert_eq!(
            verdicts
                .iter()
                .filter(|line| line.contains("  finish()  "))
                .collect::<Vec<_>>(),
            [expected],
            "{body}"
        );
    }
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn a_base_reference_uses_the_final_subtypes_override_instead_of_the_base_body() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Base::same).forEach(Base::finish);",
        "class Base { Base same() { return this; } void finish() {} } final class Builder extends Base { Base next; @Override Base same() { return next; } }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: method reference Base::finish: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac and javap; run with --ignored"]
fn a_base_reference_does_not_reselect_a_subtype_overload() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Base::same).forEach(Base::finish);",
        "class Base { Base next; Base same(Object... values) { return next; } void finish() {} } final class Builder extends Base { Builder same() { return this; } }",
    );
    assert!(
        project
            .bytecode()
            .contains("same:([Ljava/lang/Object;)Lshop/Base;")
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: method reference Base::finish: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac and javap; run with --ignored"]
fn a_base_reference_keeps_the_selected_variable_arity_declaration() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Base::same).forEach(Base::finish);",
        "class Base { Base next; Base same(Object... values) { return this; } void finish() {} } final class Builder extends Base { Base same() { return next; } }",
    );
    assert!(
        project
            .bytecode()
            .contains("same:([Ljava/lang/Object;)Lshop/Base;")
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  finish()  "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Allowed: method reference Base::finish: the object returned by static call of() is a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac and javap; run with --ignored"]
fn a_base_reference_keeps_generic_specialization_separate_from_runtime_dispatch() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Base::same).forEach(Base::finish);",
        "class Base<T> { Base<T> next; Base<T> same(T... values) { return this; } void finish() {} } final class Builder extends Base<String> { Base<String> same(Integer... values) { return next; } }",
    );
    assert!(
        project
            .bytecode()
            .contains("same:([Ljava/lang/Object;)Lshop/Base;")
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  finish()  "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Allowed: method reference Base::finish: the object returned by static call of() is a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_open_functional_receiver_does_not_gain_identity_from_a_base_reference() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Base::same).forEach(Base::finish);",
        "class Base { Base same() { return this; } void finish() {} } class Builder extends Base {}",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: method reference Base::finish: the object returned by same() is not a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_interface_reference_keeps_the_final_functional_receivers_identity() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Base::same).forEach(Base::finish);",
        "interface Base { default Base same() { return this; } void finish(); } final class Builder implements Base { public void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(0));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  finish()  "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Allowed: method reference Base::finish: the object returned by static call of() is a friend of f",
        ]
    );
}

#[test]
#[ignore = "requires a JDK with javac; run with --ignored"]
fn an_interface_reference_uses_the_more_specific_default_implementation() {
    let project = CompiledProject::scenario(
        "java.util.List.of(builder).stream().map(Base::same).forEach(Base::finish);",
        "interface Base { default Base same() { return this; } void finish(); } interface Other extends Base { default Base same() { return null; } } final class Builder implements Base, Other { public void finish() {} }",
    );

    let output = project.check("inspect");

    assert_eq!(output.status.code(), Some(1));
    let verdicts = project.verdicts(&output);
    assert_eq!(
        verdicts
            .iter()
            .filter(|line| line.contains("  Violation: "))
            .collect::<Vec<_>>(),
        [
            ":4  shop.OrderService.inspect  finish()  Violation: method reference Base::finish: the object returned by same() is not a friend of f",
        ]
    );
}

struct CompiledProject {
    temporary: tempfile::TempDir,
    source: PathBuf,
    classes: PathBuf,
}

impl CompiledProject {
    fn scenario(body: &str, types: &str) -> Self {
        Self::new(&format!(
            "package shop;\npublic class OrderService {{\n    void inspect(Builder builder) {{\n        {body}\n    }}\n}}\n{types}\n"
        ))
    }

    fn new(text: &str) -> Self {
        Self::compile(text, None)
    }

    fn compile(text: &str, lombok: Option<PathBuf>) -> Self {
        let temporary = tempfile::tempdir().expect("temporary project");
        let source = temporary.path().join("OrderService.java");
        let classes = temporary.path().join("classes");
        fs::write(&source, text).expect("fixture source");
        let mut compiler = Command::new("javac");
        if let Some(lombok) = lombok {
            compiler
                .arg("-cp")
                .arg(&lombok)
                .arg("-processorpath")
                .arg(lombok);
        }
        let compiled = compiler
            .args(["--release", "17", "-d"])
            .arg(&classes)
            .arg(&source)
            .output()
            .expect("javac runs");
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        Self {
            temporary,
            source,
            classes,
        }
    }

    fn check(&self, method: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_law-of-demeter"))
            .arg("--source")
            .arg(self.temporary.path())
            .arg("--classpath")
            .arg(&self.classes)
            .args(["--release", "17", "--show-allowed", "--only"])
            .arg(format!("shop.OrderService#{method}"))
            .output()
            .expect("the binary runs")
    }

    fn bytecode(&self) -> String {
        let output = Command::new("javap")
            .args(["-c", "-p", "-v", "-classpath"])
            .arg(&self.classes)
            .arg("shop.OrderService")
            .output()
            .expect("javap runs");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("UTF-8 bytecode listing")
    }

    fn verdicts(&self, output: &Output) -> Vec<String> {
        let report = String::from_utf8_lossy(&output.stdout);
        assert!(report.contains("Source errors: 0"), "{report}");
        assert!(report.contains("Type reading errors: 0"), "{report}");
        report
            .lines()
            .filter_map(|line| line.strip_prefix(self.source.to_str().expect("UTF-8 fixture path")))
            .map(str::to_owned)
            .collect()
    }
}
