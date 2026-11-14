use law_of_demeter_java_source::{Expression, Statement, parse};

const BROAD_SOURCE: &str = r#"
package shop.broad;

import java.util.*;
import java.util.function.Function;
import static java.lang.Math.max;

@interface Marker { String value() default ""; }

enum Color implements Named {
    RED(1), GREEN(2) { @Override void paint() { helper.go(); } };
    private final int code;
    Color(int code) { this.code = code; }
    void paint() { }
}

interface Shape<T extends Comparable<T>> extends Base, Other {
    <R> R accept(Function<? super T, ? extends R> visitor);
    default void draw() { render(); }
    static int count() { return 0; }
}

record Pair<A, B>(A first, B second) implements Comparable<Pair<A, B>> {
    static int created;
    Pair { Objects.requireNonNull(first); }
    public int compareTo(Pair<A, B> other) { return 0; }
}

@SuppressWarnings({"unchecked", "rawtypes"})
public abstract class Broad<T> extends Base<T> implements Runnable {
    static { init.run(); }
    { setup.run(); }
    private final Map<String, List<? extends Number>> index = new HashMap<>();
    int[] numbers = {1, 2, 3}, more[];

    protected Broad(int x) { super(x); this.numbers = new int[x]; }
    Broad() { this(1); }

    abstract void work() throws Exception;

    public synchronized <U extends Number & Comparable<U>> List<U> convert(final Collection<? extends U> input, U... rest) {
        Runnable r = new Runnable() { public void run() { inner.go(); } };
        class Local { void m() { hidden.go(); } }
        Object o = (Runnable & java.io.Serializable) () -> input.size();
        var text = """
            Hello "quoted" \
            world
            """;
        char c = 'A';
        long big = 0xFFFF_FFFFL;
        double d = 1e-3 + .5f;
        int result = input == null ? -1 : input.size() > 3 ? ~input.size() : +input.size();
        for (var entry : index.entrySet()) { entry.getValue().forEach(System.out::println); }
        for (int i = 0, j = 10; i < j; i++, j--) { continue; }
        outer:
        for (;;) { break outer; }
        do { result--; } while (result > 0);
        try (var in = open(); var out = open()) { in.transferTo(out); }
        catch (java.io.IOException | RuntimeException e) { throw new IllegalStateException(e); }
        finally { cleanup.run(); }
        switch (o) {
            case String s when s.isEmpty() -> s.length();
            case Pair(var a, var b) -> a.hashCode();
            case null, default -> { }
        }
        int[][] grid = new int[3][];
        String joined = String.join(",", List.of("a", "b"));
        assert result > 0 : "positive";
        if (o instanceof Pair<?, ?> p && p.first() != null) { p.second(); }
        this.<String>generic("x");
        Broad.this.work();
        return null;
    }

    void work2() { max(1, 2); }
    class Inner { class Deeper { void go() { index.clear(); } } }
}
"#;

#[test]
fn a_file_with_many_kinds_of_declarations_and_statements_parses() {
    let file = parse("Broad.java", BROAD_SOURCE).unwrap();

    let names: Vec<&str> = file
        .classes
        .iter()
        .map(|class| class.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "shop.broad.Marker",
            "shop.broad.Color",
            "shop.broad.Color$1",
            "shop.broad.Shape",
            "shop.broad.Pair",
            "shop.broad.Broad",
            "shop.broad.Broad$1",
            "shop.broad.Broad$1Local",
            "shop.broad.Broad$Inner",
            "shop.broad.Broad$Inner$Deeper"
        ]
    );
}

#[test]
fn a_field_of_an_outer_class_is_named_with_its_owner_from_a_deeper_class() {
    let file = parse("Broad.java", BROAD_SOURCE).unwrap();

    let deeper = file.classes.last().unwrap();
    let Some(Statement::Expression(Expression::Call(call))) = deeper.methods[0]
        .body
        .as_ref()
        .and_then(|body| body.first())
    else {
        panic!("expected a call");
    };
    assert_eq!(
        call.receiver.as_deref(),
        Some(&Expression::OuterField {
            owner: "shop.broad.Broad".to_owned(),
            name: "index".to_owned()
        })
    );
}

#[test]
fn the_calls_around_an_anonymous_class_body_are_kept() {
    let source = "class A { void f() { new Base() { void m() { hidden.go(); } }.run(); } }";

    let file = parse("A.java", source).unwrap();

    let Some(Statement::Expression(Expression::Call(call))) = file.classes[0].methods[0]
        .body
        .as_ref()
        .and_then(|body| body.first())
    else {
        panic!("expected a call");
    };
    assert_eq!(call.name, "run");
    assert!(matches!(call.receiver.as_deref(), Some(Expression::New(_))));
}

#[test]
fn non_ascii_text_and_windows_line_endings_keep_lines_right() {
    let source = "class Sipariş {\r\n  String ad = \"çğıöşü 😀\";\r\n  void öde(Müşteri müşteri) {\r\n    müşteri.cüzdan().çek();\r\n  }\r\n}";

    let file = parse("Sipariş.java", source).unwrap();

    let pay = &file.classes[0].methods[0];
    assert_eq!(pay.name, "öde");
    assert_eq!(pay.line, 3);
    let Some(Statement::Expression(Expression::Call(call))) =
        pay.body.as_ref().and_then(|body| body.first())
    else {
        panic!("expected a call");
    };
    assert_eq!(call.name, "çek");
    assert_eq!(call.line, 4);
}

#[test]
fn comments_between_tokens_do_not_change_the_model() {
    let source = "class A {\n  void f(Order /* the order */ order) {\n    // first\n    order /* x */ .go(/* none */); /* y */\n    /** doc */\n  }\n}";

    let file = parse("A.java", source).unwrap();

    let method = &file.classes[0].methods[0];
    let Some([Statement::Expression(Expression::Call(call))]) = method.body.as_deref() else {
        panic!("expected one call, got {:?}", method.body);
    };
    assert_eq!(call.name, "go");
    assert_eq!(
        call.receiver.as_deref(),
        Some(&Expression::Parameter("order".to_owned()))
    );
    assert!(call.arguments.is_empty());
}

#[test]
fn broken_sources_are_errors_and_never_panics() {
    let broken = [
        "class",
        "class A {",
        "class A { void f( { } }",
        "class A { void f() { int = 3; } }",
        "}}}{{{",
        "class A { void f() { order.(); } }",
        "class A { void f() { \"unterminated } }",
        "\u{0}\u{1}",
        "package ;",
        "import ;",
    ];

    for source in broken {
        let error = parse("Broken.java", source).unwrap_err();

        assert_eq!(error.file, "Broken.java", "source: {source:?}");
        assert!(error.line >= 1, "source: {source:?}");
    }
}
