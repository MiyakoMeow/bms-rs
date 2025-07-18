use bms_rs::lex::{BmsLexOutput, parse, token::Token};

#[test]
fn test_comment() {
    let text = r"
    #Comment This is a comment
    This is another comment
    This is the third comment💖

    This is the fourth comment";

    let BmsLexOutput {
        tokens,
        lex_warnings: warnings,
    } = parse(text);
    assert_eq!(warnings, vec![]);
    let mut ts_iter = tokens.into_iter();
    assert_eq!(ts_iter.next(), Some(Token::Comment("This is a comment")));
    assert_eq!(
        ts_iter.next(),
        Some(Token::NotACommand("This is another comment"))
    );
    assert_eq!(
        ts_iter.next(),
        Some(Token::NotACommand("This is the third comment💖"))
    );
    assert_eq!(
        ts_iter.next(),
        Some(Token::NotACommand("This is the fourth comment"))
    );
    assert_eq!(ts_iter.next(), None);
}
