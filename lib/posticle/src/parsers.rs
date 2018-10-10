use std::cell::Cell;

use nom::{digit1, IResult};
use unic_ucd_category::GeneralCategory as GCat;

fn is_invalid_char(c: char) -> bool {
    match c as u32 {
        0xFFFE => true,
        0xFEFF => true,
        0xFFFF => true,
        0x202A..=0x202E => true,
        _ => false,
    }
}

fn is_punctuation_no_hyphen_underscore(c: char) -> bool {
    match c {
        '!' => true,
        '"' => true,
        '#' => true,
        '$' => true,
        '%' => true,
        '&' => true,
        '\'' => true,
        '(' => true,
        ')' => true,
        '*' => true,
        '+' => true,
        ',' => true,
        '.' => true,
        '/' => true,
        ':' => true,
        ';' => true,
        '<' => true,
        '=' => true,
        '>' => true,
        '?' => true,
        '@' => true,
        '[' => true,
        ']' => true,
        '^' => true,
        '`' => true,
        '{' => true,
        '|' => true,
        '}' => true,
        '~' => true,
        _ => false,
    }
}

fn is_punctuation(c: char) -> bool {
    match c {
        '-' => true,
        '_' => true,
        c => is_punctuation_no_hyphen_underscore(c),
    }
}

fn is_valid_domain_extrema_char(c: char) -> bool {
    !(is_punctuation(c) || c.is_ascii_control() || is_invalid_char(c) || c.is_whitespace())
}

fn is_valid_domain_middle_char(c: char) -> bool {
    !(is_punctuation_no_hyphen_underscore(c)
        || c.is_ascii_control()
        || is_invalid_char(c)
        || c.is_whitespace())
}

fn valid_domain_segment(i: &str) -> IResult<&str, &str> {
    // The first and last characters can't be a hyphen or underscore.
    // valid_domain_middle char matches hyphens and underscores, so we can manually check if we
    // haven't seen a character yet or if we're done.
    let prev: Cell<Option<char>> = Cell::new(None);
    verify!(
        i,
        take_while!(|c| prev
            .replace(Some(c))
            .map_or_else(|| c == '-' || c == '_', |_| true)
            && is_valid_domain_middle_char(c)),
        |_| prev.get().filter(|&c| !(c == '-' || c == '_')).is_some()
    )
}

named!(syntactically_valid_tld<&str, &str>, recognize!(do_parse!(
    take_while_m_n!(0, 1, is_valid_domain_extrema_char) >>
    take_while_m_n!(1, 1, |c| {
        !(is_punctuation(c)
            || c.is_ascii_control()
            || is_invalid_char(c)
            || c.is_whitespace()
            || c.is_ascii_digit())
    }) >>
    opt!(alt!(
        recognize!(tuple!(
            take_while_m_n!(1, 60, is_valid_domain_middle_char),
            take_while_m_n!(1, 1, is_valid_domain_extrema_char)
        )) |
        take_while_m_n!(1, 1, is_valid_domain_extrema_char)
    )) >>
    ()
)));

named!(valid_domain<&str, &str>, recognize!(tuple!(
    separated_nonempty_list!(char!('.'), valid_domain_segment),
    char!('.'),
    syntactically_valid_tld
)));

fn is_valid_path_char(c: char) -> bool {
    match c {
        '(' => true,
        ')' => true,
        '<' => true,
        '>' => true,
        '?' => true,
        c if c.is_whitespace() => true,
        _ => false,
    }
}

fn is_valid_path_ending_char(c: char) -> bool {
    match c {
        '!' => true,
        '"' => true,
        '$' => true,
        '%' => true,
        '&' => true,
        '*' => true,
        ',' => true,
        '.' => true,
        ':' => true,
        ';' => true,
        '=' => true,
        '@' => true,
        '[' => true,
        '\'' => true,
        ']' => true,
        '|' => true,
        '~' => true,
        c => is_valid_path_char(c),
    }
}

named!(valid_path_ending<&str, &str>,
    alt!(take_while_m_n!(1, 1, is_valid_path_ending_char) | valid_path_balanced_parens)
);

named!(valid_path_balanced_parens<&str, &str>, delimited!(
    char!('('),
    alt!(take_while1!(is_valid_path_char) |
         delimited!(char!('('), take_while1!(is_valid_path_char), char!(')'))),
    char!(')')
));

named!(valid_path_segment<&str, &str>, alt!(
    recognize!(do_parse!(
        take_while1!(is_valid_path_char) >>
        many0!(tuple!(valid_path_balanced_parens, take_while!(is_valid_path_char))) >>
        valid_path_ending >>
        ()
    )) | recognize!(terminated!(take_while!(is_valid_path_char), char!('/')))
));

fn is_valid_query_string_char(c: char) -> bool {
    match c {
        '!' => true,
        '#' => true,
        '$' => true,
        '%' => true,
        '&' => true,
        '(' => true,
        ')' => true,
        '*' => true,
        '+' => true,
        ',' => true,
        '-' => true,
        '.' => true,
        '/' => true,
        ':' => true,
        ';' => true,
        '=' => true,
        '?' => true,
        '@' => true,
        '[' => true,
        '\'' => true,
        ']' => true,
        '_' => true,
        '|' => true,
        '~' => true,
        c if c.is_ascii_alphanumeric() => true,
        _ => false,
    }
}

fn is_valid_query_string_end_char(c: char) -> bool {
    match c {
        '#' => true,
        '&' => true,
        '-' => true,
        '/' => true,
        '=' => true,
        '_' => true,
        c if c.is_ascii_alphanumeric() => true,
        _ => false,
    }
}

named!(valid_query_string<&str, &str>, recognize!(tuple!(
    take_while!(is_valid_query_string_char),
    take_while_m_n!(1, 1, is_valid_query_string_end_char)
)));

named!(pub valid_url<&str, &str>, recognize!(do_parse!(
    tag!("http") >> opt!(char!('s')) >> tag!("://") >>
    valid_domain >>
    opt!(preceded!(char!(':'), digit1)) >>
    opt!(preceded!(char!('/'), valid_path_segment)) >>
    opt!(preceded!(char!('?'), valid_query_string)) >>
    ()
)));

// TODO figure out what general categories these are in
fn is_hashtag_special_char(c: char) -> bool {
    match c {
        '_' => true,
        '\u{00b7}' => true, // MIDDLE DOT
        '\u{05be}' => true, // HEBREW PUNCTUATION MAQAF
        '\u{05f3}' => true, // HEBREW PUNCTUATION GERESH
        '\u{05f4}' => true, // HEBREW PUNCTUATION GERSHAYIM
        '\u{0f0b}' => true, // TIBETAN MARK INTERSYLLABIC TSHEG
        '\u{0f0c}' => true, // TIBETAN MARK DELIMITER TSHEG BSTAR
        '\u{200c}' => true, // ZERO WIDTH NON-JOINER
        '\u{200d}' => true, // ZERO WIDTH JOINER
        '\u{3003}' => true, // DITTO MARK
        '\u{301c}' => true, // WAVE DASH
        '\u{309b}' => true, // KATAKANA-HIRAGANA VOICED SOUND MARK
        '\u{309c}' => true, // KATAKANA-HIRAGANA SEMI-VOICED SOUND MARK
        '\u{30a0}' => true, // KATAKANA-HIRAGANA DOUBLE HYPHEN
        '\u{30fb}' => true, // KATAKANA MIDDLE DOT
        '\u{a67e}' => true, // CYRILLIC KAVYKA
        '\u{ff5e}' => true, // FULLWIDTH TILDE
        _ => false,
    }
}

pub fn hashtag<'a>(i: &'a str) -> IResult<&'a str, &'a str> {
    // Cells of parsed characters are used here due to the lack of backtracking.
    let prev1 = Cell::new(None);
    let prev2 = Cell::new(None);
    preceded!(
        i,
        char!('#'),
        verify!(
            take_while!(|c| {
                prev2.replace(prev1.replace(Some(c)));
                is_hashtag_special_char(c) || {
                    let cat = GCat::of(c);
                    GCat::is_letter(&cat) || cat == GCat::DecimalNumber || GCat::is_mark(&cat)
                }
            }),
            |_| prev2
                .get()
                .filter(|c| {
                    let cat = GCat::of(*c);
                    GCat::is_letter(&cat) || GCat::is_mark(&cat)
                })
                .is_some()
        )
    )
}

named!(pub mention<&str, (&str, Option<&str>)>, do_parse!(
    char!('@') >>
    username: take_while_m_n!(1, 32, |c: char| c.is_ascii_alphanumeric()) >>
    domain_: opt!(preceded!(char!('@'), valid_domain)) >>
    (username, domain_)
));
