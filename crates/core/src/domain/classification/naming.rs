const FACTORY_WORDS: [&str; 5] = ["create", "new", "of", "from", "build"];

const QUERY_WORDS: [&str; 3] = ["find", "load", "fetch"];

const REPOSITORY_QUERY_WORDS: [&str; 1] = ["get"];

const REPOSITORY_SUFFIX: &str = "Repository";

pub fn is_factory_method(name: &str) -> bool {
    starts_with_word(name, &FACTORY_WORDS)
}

pub fn is_repository_query(name: &str) -> bool {
    starts_with_word(name, &QUERY_WORDS)
}

pub fn is_query_on_repository(owner: &str, name: &str) -> bool {
    owner.ends_with(REPOSITORY_SUFFIX) && starts_with_word(name, &REPOSITORY_QUERY_WORDS)
}

fn starts_with_word(name: &str, words: &[&str]) -> bool {
    words.iter().any(|word| {
        name.strip_prefix(word)
            .is_some_and(|rest| !rest.starts_with(char::is_lowercase))
    })
}
