use std::ffi::OsString;
use std::path::PathBuf;

use law_of_demeter_core::application::Target;

use super::{Arguments, Command, UsageError, parse, usage};

#[test]
fn rust_options_and_opaque_selections_work_in_any_order() {
    let arguments = [
        "--only=path+file:///project#shop@0.1.0::shop::Order#pay",
        "--features=payments,audit",
        "--features",
        "receipts",
        "--package=shop",
        "--no-default-features",
        "--target=aarch64-unknown-linux-gnu",
        "--target-dir=output with spaces",
        "--rust",
        "project with spaces/Cargo.toml",
        "--show-allowed",
    ];

    let command = parse(arguments.into_iter().map(OsString::from));

    assert!(matches!(command, Ok(Command::Check(_))), "{command:?}");
    let Ok(Command::Check(arguments)) = command else {
        unreachable!()
    };
    let rust = arguments.rust.expect("Rust options");
    assert_eq!(
        rust.manifest,
        PathBuf::from("project with spaces/Cargo.toml")
    );
    assert_eq!(rust.package, "shop");
    assert_eq!(rust.features, ["payments", "audit", "receipts"]);
    assert!(rust.no_default_features);
    assert_eq!(rust.target.as_deref(), Some("aarch64-unknown-linux-gnu"));
    assert_eq!(rust.target_dir, Some(PathBuf::from("output with spaces")));
    assert_eq!(
        rust.targets,
        ["path+file:///project#shop@0.1.0::shop::Order#pay"]
    );
    assert!(arguments.show_allowed);
}

#[test]
fn rust_requires_an_explicit_package() {
    let error = rejected(&["--rust", "Cargo.toml"]);

    assert_eq!(error.0, "--rust requires --package <NAME>");
}

#[test]
fn rust_rejects_java_options_in_either_order() {
    let mixed = [
        "--source=src",
        "--classpath=",
        "--jdk-home=/jdk",
        "--release=21",
    ]
    .map(|option| {
        [
            rejected(&[option, "--rust=Cargo.toml", "--package=shop"]),
            rejected(&["--rust=Cargo.toml", "--package=shop", option]),
        ]
    });

    for errors in mixed {
        for error in errors {
            assert_eq!(error.0, "--rust cannot be combined with Java options");
        }
    }
}

#[test]
fn rust_configuration_requires_rust_selection() {
    let errors = [
        "--package=shop",
        "--features=payments",
        "--no-default-features",
        "--target=host",
        "--target-dir=output",
    ]
    .map(|option| rejected(&[option]));

    for error in errors {
        assert_eq!(error.0, "Rust options require --rust <Cargo.toml>");
    }
}

#[test]
fn a_following_option_is_not_consumed_as_a_missing_value() {
    let error = rejected(&["--rust", "--package=shop"]);

    assert_eq!(error.0, "--rust needs a value");
}

fn parsed(arguments: &[&str]) -> Command {
    parse(arguments.iter().map(OsString::from)).expect("arguments parse")
}

#[test]
fn source_roots_can_be_repeated_in_long_and_short_form() {
    let command = parsed(&["--source", "src/main/java", "-s", "src/test/java"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![
                PathBuf::from("src/main/java"),
                PathBuf::from("src/test/java")
            ],
            ..Arguments::default()
        })
    );
}

#[cfg(unix)]
#[test]
fn classpath_entries_can_be_repeated_and_joined_with_colons() {
    let command = parsed(&[
        "-s",
        "src",
        "--classpath",
        "a.jar::lib/classes",
        "-c",
        "b.jar",
    ]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("src")],
            classpath: vec![
                PathBuf::from("a.jar"),
                PathBuf::from("lib/classes"),
                PathBuf::from("b.jar")
            ],
            ..Arguments::default()
        })
    );
}

#[test]
fn the_jdk_home_and_the_release_are_read() {
    let command = parsed(&["-s", "src", "--jdk-home", "/opt/jdk-21", "--release", "17"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("src")],
            jdk_home: Some(PathBuf::from("/opt/jdk-21")),
            release: Some(17),
            ..Arguments::default()
        })
    );
}

#[test]
fn show_allowed_is_a_flag() {
    let command = parsed(&["-s", "src", "--show-allowed"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("src")],
            show_allowed: true,
            ..Arguments::default()
        })
    );
}

fn rejected(arguments: &[&str]) -> UsageError {
    parse(arguments.iter().map(OsString::from)).expect_err("arguments are rejected")
}

#[test]
fn the_source_roots_may_be_left_out() {
    let command = parsed(&["--classpath", "lib"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            classpath: vec![PathBuf::from("lib")],
            ..Arguments::default()
        })
    );
}

#[test]
fn an_unknown_option_is_rejected() {
    let error = rejected(&["-s", "src", "--verbose"]);

    assert_eq!(error, UsageError("unknown option: --verbose".to_owned()));
}

#[test]
fn an_option_without_its_value_is_rejected() {
    let error = rejected(&["-s", "src", "--classpath"]);

    assert_eq!(error, UsageError("--classpath needs a value".to_owned()));
}

#[test]
fn a_release_that_is_not_a_number_is_rejected() {
    let error = rejected(&["-s", "src", "--release", "twenty"]);

    assert_eq!(
        error,
        UsageError("--release must be a version number: twenty".to_owned())
    );
}

#[test]
fn a_stray_positional_argument_is_rejected() {
    let error = rejected(&["-s", "src", "Order.java"]);

    assert_eq!(
        error,
        UsageError("unexpected argument: Order.java".to_owned())
    );
}

#[test]
fn an_option_can_carry_its_value_after_an_equals_sign() {
    let command = parsed(&["--source=src", "--release=21", "--jdk-home=/opt/jdk"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("src")],
            jdk_home: Some(PathBuf::from("/opt/jdk")),
            release: Some(21),
            ..Arguments::default()
        })
    );
}

#[test]
fn an_inline_java_path_keeps_its_leading_dash() {
    let inline = parsed(&["--source=-src", "--classpath=out"]);
    let relative = parsed(&["--source=./-src", "--classpath=out"]);

    assert_eq!(
        inline,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("-src")],
            classpath: vec![PathBuf::from("out")],
            ..Arguments::default()
        })
    );
    assert_eq!(
        relative,
        Command::Check(Arguments {
            sources: vec![PathBuf::from("./-src")],
            classpath: vec![PathBuf::from("out")],
            ..Arguments::default()
        })
    );
}

#[test]
fn inline_rust_paths_keep_their_leading_dashes() {
    let command = parsed(&[
        "--rust=-library/Cargo.toml",
        "--package=shop",
        "--target-dir=-analysis",
        "--features=payment",
    ]);

    let Command::Check(arguments) = command else {
        panic!("check command")
    };
    let rust = arguments.rust.expect("Rust configuration");
    assert_eq!(rust.manifest, PathBuf::from("-library/Cargo.toml"));
    assert_eq!(rust.target_dir, Some(PathBuf::from("-analysis")));
    assert_eq!(rust.features, ["payment"]);
}

#[test]
fn help_is_requested_with_either_spelling_and_needs_no_source() {
    let commands = [parsed(&["--help"]), parsed(&["-h"])];

    assert_eq!(commands, [Command::Help, Command::Help]);
}

#[test]
fn the_usage_text_names_every_option() {
    let text = usage();

    let missing: Vec<_> = [
        "--source",
        "--classpath",
        "--jdk-home",
        "--release",
        "--show-allowed",
        "--only",
        "--skill",
        "update",
        "--help",
        "--rust",
        "--package",
        "--features",
        "--no-default-features",
        "--target",
        "--target-dir",
    ]
    .into_iter()
    .filter(|option| !text.contains(option))
    .collect();

    assert_eq!(missing, Vec::<&str>::new());
}

#[test]
fn classes_and_methods_can_be_selected_more_than_once() {
    let command = parsed(&["--only", "shop.Order", "--only=shop.Customer#getWallet"]);

    assert_eq!(
        command,
        Command::Check(Arguments {
            targets: vec![
                Target::class("shop.Order"),
                Target::method("shop.Customer", "getWallet")
            ],
            ..Arguments::default()
        })
    );
}

#[test]
fn a_target_without_a_class_or_a_method_name_is_rejected() {
    let errors = [
        rejected(&["--only", "#pay"]),
        rejected(&["--only", "shop.Order#"]),
    ];

    assert_eq!(
        errors,
        [
            UsageError("--only needs a class, optionally followed by #method: #pay".to_owned()),
            UsageError(
                "--only needs a class, optionally followed by #method: shop.Order#".to_owned()
            )
        ]
    );
}

#[test]
fn a_target_with_more_than_one_method_separator_is_rejected() {
    let error = rejected(&["--only", "shop.Order#pay#now"]);

    assert_eq!(
        error,
        UsageError(
            "--only needs a class, optionally followed by #method: shop.Order#pay#now".to_owned()
        )
    );
}
