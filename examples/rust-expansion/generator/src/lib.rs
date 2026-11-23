use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

#[proc_macro]
pub fn generated_charge(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter();
    let (Some(TokenTree::Ident(receiver)), None) = (input.next(), input.next()) else {
        return compile_error("expected one receiver identifier");
    };
    let charge = Group::new(
        Delimiter::Parenthesis,
        TokenTree::Literal(Literal::i32_unsuffixed(13)).into(),
    );
    let output = [
        TokenTree::Ident(receiver.clone()),
        Punct::new('.', Spacing::Alone).into(),
        Ident::new("charge", Span::call_site()).into(),
        charge.into(),
        Punct::new(';', Spacing::Alone).into(),
        receiver.into(),
        Punct::new('.', Spacing::Alone).into(),
        Ident::new("balance", Span::call_site()).into(),
        Group::new(Delimiter::Parenthesis, TokenStream::new()).into(),
    ];

    TokenTree::Group(Group::new(Delimiter::Brace, output.into_iter().collect())).into()
}

fn compile_error(message: &str) -> TokenStream {
    [
        TokenTree::Ident(Ident::new("compile_error", Span::call_site())),
        Punct::new('!', Spacing::Alone).into(),
        Group::new(
            Delimiter::Parenthesis,
            TokenTree::Literal(Literal::string(message)).into(),
        )
        .into(),
    ]
    .into_iter()
    .collect()
}
