extern crate pom;

use pom::parser::*;
use pom::Parser;

use std::iter::FromIterator;

#[derive(Debug, PartialEq)]
enum TextPart {
    Str(String),
    Tag(String),
}

#[derive(Debug, PartialEq)]
enum ParsedPart {
    Char(char),
    Tag(String),
}

fn phrase_start() -> Parser<char, usize> {
    let hashes = sym('#').repeat(1..);
    let paren = sym('(');
    let start = hashes - paren;
    start.map(|h| h.len())
}

fn phrase_end(len: usize) -> Parser<char, ()> {
    let hashes = sym('#').repeat(len..len + 1);
    let paren = sym(')');
    (paren - hashes).discard()
}

fn phrase_content_char<F>(make_end_parser: F) -> Parser<char, char>
where
    F: Fn() -> Parser<char, ()>,
{
    let end_parser = make_end_parser();
    !end_parser * (take(1).map(|cs| cs[0]))
}

// fn phrase_content_until(end_parser: Parser<char, ()>) -> Parser<char, String> {
fn phrase_content_until<F>(make_end_parser: F) -> Parser<char, String>
where
    F: Fn() -> Parser<char, ()>,
{
    let end_parser = make_end_parser();
    (phrase_content_char(make_end_parser).repeat(1..) - end_parser)
        .map(|chars| String::from_iter(chars))
}

fn phrase_hash() -> Parser<char, String> {
    phrase_start() >> (|c| phrase_content_until(|| phrase_end(c)))
}

fn char_as_parsed_part() -> Parser<char, ParsedPart> {
    take(1).map(|c: &[char]| ParsedPart::Char(c[0]))
}

fn phrase_hash_as_parsed_part() -> Parser<char, ParsedPart> {
    phrase_hash().map(ParsedPart::Tag)
}

fn char_or_hash() -> Parser<char, ParsedPart> {
    phrase_hash_as_parsed_part() | char_as_parsed_part()
}

fn parsed_parts() -> Parser<char, Vec<ParsedPart>> {
    char_or_hash().repeat(0..)
}

enum CollectedPart {
    Chars(Vec<char>),
    Tag(String),
}

fn collected_parts() -> Parser<char, Vec<CollectedPart>> {
    parsed_parts().map(|pps| {
        let mut cps: Vec<CollectedPart> = Vec::with_capacity(pps.len() / 4); // guess!
        let mut current: Vec<char> = Vec::new();
        for pp in pps {
            match pp {
                ParsedPart::Char(c) => current.push(c),
                ParsedPart::Tag(t) => {
                    if !current.is_empty() {
                        cps.push(CollectedPart::Chars(current.clone()));
                        current.truncate(0);
                    }
                    cps.push(CollectedPart::Tag(t));
                }
            }
        }
        if !current.is_empty() {
            cps.push(CollectedPart::Chars(current.clone()));
        }
        cps
    })
}

fn text_parts() -> Parser<char, Vec<TextPart>> {
    collected_parts().map(|parts| {
        parts
            .iter()
            .map(|cp| match cp {
                CollectedPart::Chars(chars) => TextPart::Str(chars.into_iter().collect()),
                CollectedPart::Tag(tag) => TextPart::Tag(tag.to_string()),
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrase_start_matches_one() {
        let parser = phrase_start();
        let output = parser.parse(&['#', '(', 'f', 'o', 'o']);
        assert_eq!(output, Ok(1))
    }

    #[test]
    fn phrase_start_matches_many() {
        let parser = phrase_start();
        let output = parser.parse(&['#', '#', '#', '(', 'f', 'o', 'o']);
        assert_eq!(output, Ok(3))
    }

    #[test]
    fn phrase_end_matches_one() {
        let output = phrase_end(1).parse(&[')', '#', '#']);
        assert!(output.is_ok());
    }

    #[test]
    fn phrase_end_matches_many() {
        let output = phrase_end(2).parse(&[')', '#', '#']);
        assert!(output.is_ok());
    }

    #[test]
    fn phrase_end_requires_enough() {
        let output = phrase_end(3).parse(&[')', '#', '#']);
        assert!(output.is_err());
    }

    #[test]
    fn phrase_content_char_takes_one() {
        let output = phrase_content_char(|| phrase_end(1)).parse(&['o', 'p', ')', '#']);
        assert_eq!(output, Ok('o'));
    }

    #[test]
    fn phrase_content_until_collects_all() {
        let output = phrase_content_until(|| phrase_end(1)).parse(&['#', '(', 'a', 'b', ')', '#']);
        assert_eq!(output, Ok("#(ab".to_owned()));
    }

    #[test]
    fn phrase_hash_matches_one() {
        let output = phrase_hash().parse(&['#', '(', 'a', 'b', ')', '#']);
        assert_eq!(output, Ok("ab".to_owned()));
    }

    #[test]
    fn phrase_hash_matches_two() {
        let output = phrase_hash().parse(&['#', '#', '(', 'b', 'c', ')', '#', '#']);
        assert_eq!(output, Ok("bc".to_owned()));
    }

    #[test]
    fn text_parts_collects_all() {
        let output =
            text_parts().parse(&['a', 'b', ' ', '#', '#', '(', 'c', 'd', ')', '#', '#', 'z']);
        assert_eq!(
            output,
            Ok(vec![
                TextPart::Str("ab ".to_string()),
                TextPart::Tag("cd".to_string()),
                TextPart::Str("z".to_string())
            ])
        )
    }
}
