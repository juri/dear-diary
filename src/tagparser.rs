extern crate pom;

use pom::parser::*;

use std::iter::FromIterator;

pub fn find_tags(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let res = text_parts().parse(&chars);
    let mut tags = res
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| match p {
                    TextPart::Str(_) => None,
                    TextPart::Tag(t) => Some(t.to_string()),
                })
                .collect()
        })
        .unwrap_or(vec![]);
    tags.sort();
    tags.dedup();
    tags
}

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

fn phrase_start<'a>() -> Parser<'a, char, usize> {
    let hashes = sym('#').repeat(1..);
    let paren = sym('(');
    let start = hashes - paren;
    start.map(|h| h.len())
}

fn phrase_end<'a>(len: usize) -> Parser<'a, char, ()> {
    let hashes = sym('#').repeat(len..len + 1);
    let paren = sym(')');
    (paren - hashes).discard()
}

fn phrase_content_char<'a, F>(make_end_parser: F) -> Parser<'a, char, char>
where
    F: Fn() -> Parser<'a, char, ()>,
{
    let end_parser = make_end_parser();
    !end_parser * (take(1).map(|cs| cs[0]))
}

// fn phrase_content_until(end_parser: Parser<'a, char, ()>) -> Parser<'a, char, String> {
fn phrase_content_until<'a, F>(make_end_parser: F) -> Parser<'a, char, String>
where
    F: Fn() -> Parser<'a, char, ()>,
{
    let end_parser = make_end_parser();
    (phrase_content_char(make_end_parser).repeat(1..) - end_parser)
        .map(|chars| String::from_iter(chars))
}

fn phrase_hash<'a>() -> Parser<'a, char, String> {
    phrase_start() >> (|c| phrase_content_until(|| phrase_end(c)))
}

fn word_hash<'a>() -> Parser<'a, char, String> {
    (sym('#') * (is_a(|c: char| c.is_alphanumeric()).repeat(1..)))
        .map(|chars| chars.into_iter().collect())
}

fn word_hash_as_parsed_part<'a>() -> Parser<'a, char, ParsedPart> {
    word_hash().map(ParsedPart::Tag)
}

fn char_as_parsed_part<'a>() -> Parser<'a, char, ParsedPart> {
    take(1).map(|c: &[char]| ParsedPart::Char(c[0]))
}

fn phrase_hash_as_parsed_part<'a>() -> Parser<'a, char, ParsedPart> {
    phrase_hash().map(ParsedPart::Tag)
}

fn char_or_hash<'a>() -> Parser<'a, char, ParsedPart> {
    phrase_hash_as_parsed_part() | word_hash_as_parsed_part() | char_as_parsed_part()
}

fn parsed_parts<'a>() -> Parser<'a, char, Vec<ParsedPart>> {
    char_or_hash().repeat(0..)
}

enum CollectedPart {
    Chars(Vec<char>),
    Tag(String),
}

fn collected_parts<'a>() -> Parser<'a, char, Vec<CollectedPart>> {
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

fn text_parts<'a>() -> Parser<'a, char, Vec<TextPart>> {
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
    fn word_hash_matches() {
        let output = word_hash().parse(&['#', 'a', 's', 'd', 'f']);
        assert_eq!(output, Ok("asdf".to_string()))
    }

    #[test]
    fn text_parts_collects_all() {
        let output = text_parts().parse(&[
            'a', 'b', ' ', '#', '#', '(', 'c', 'd', ')', '#', '#', '#', 'z', ' ', 'q', 'w',
        ]);
        assert_eq!(
            output,
            Ok(vec![
                TextPart::Str("ab ".to_string()),
                TextPart::Tag("cd".to_string()),
                TextPart::Tag("z".to_string()),
                TextPart::Str(" qw".to_string())
            ])
        )
    }

    #[test]
    fn find_tags_finds_tags() {
        let tags = find_tags("hello #world this is a #(phrase tag)#, whee");
        assert_eq!(tags, vec!["phrase tag", "world"]);
    }

    #[test]
    fn find_tags_deduplicates_tags() {
        let tags = find_tags("#a #b #a #b #c #a #c");
        assert_eq!(tags, vec!["a", "b", "c"]);
    }
}
