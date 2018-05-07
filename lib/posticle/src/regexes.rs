#![allow(dead_code)]

use regex::{Regex, RegexBuilder};

static CTRL_CHARS: &str = r"\x00-\x1F\x7F";
static INVALID_CHARS: &str = "\u{FFFE}\u{FEFF}\u{FFFF}\u{202A}-\u{202E}";
static UNICODE_SPACES: &str = "\u{09}-\u{0D}\u{20}\u{85}\u{A0}\u{1680}\u{180E}\u{2000}-\u{200A}\u{2028}\u{2029}\u{202F}\u{205F}\u{3000}";
static LATIN_ACCENTS: &str = "\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{00FF}\u{0100}-\u{024F}\u{0253}-\u{0254}\
    \u{0256}-\u{0257}\u{0259}\u{025b}\u{0263}\u{0268}\u{026F}\u{0272}\u{0289}\u{02BB}\u{1E00}-\u{1EFF}";
static PUNCTUATION: &str = r##"-_!"#$%&'()*+,./:;<=>?@\[\]^`{|}~"##;
static PUNCTUATION_NO_HYPHEN: &str = r##"_!"#$%&'()*+,./:;<=>?@\[\]^`{|}~"##;
static PUNCTUATION_NO_HYPHEN_UNDERSCORE: &str = r##"!"#$%&'()*+,./:;<=>?@\[\]^`{|}~"##;
static VALID_QUERY_STRING: &str =
    r##"(?:[-a-zA-Z0-9!?*'\(\);:&=+$/%#\[\]_\.,~|@]*[a-zA-Z0-9_&=#/])"##;

lazy_static! {
    /// Matches characters which can validly start or end a domain segment.
    static ref VALID_DOMAIN_EXTREMA_CHARS: String = format!(
        "[^{punctuation}{ctrl}{invalid}{space}]",
        punctuation = PUNCTUATION,
        ctrl = CTRL_CHARS,
        invalid = INVALID_CHARS,
        space = UNICODE_SPACES
    );

    /// Matches characters which are valid in the middle of a domain segment.
    static ref VALID_DOMAIN_MIDDLE_CHARS: String = format!(
        "[^{punctuation}{ctrl}{invalid}{space}]",
        punctuation = PUNCTUATION_NO_HYPHEN,
        ctrl = CTRL_CHARS,
        invalid = INVALID_CHARS,
        space = UNICODE_SPACES
    );

    /// Matches a valid domain segment.
    static ref VALID_DOMAIN_SEGMENT: String = format!(
        "(?:{extremum}(?:{middle}*{extremum})?)",
        extremum=*VALID_DOMAIN_EXTREMA_CHARS,
        middle=*VALID_DOMAIN_MIDDLE_CHARS,
    );

    /// Matches things that _look_ like TLDs (as per RFC 3696).
    static ref SYNTACTICALLY_VALID_TLD: String = format!(
        r"(?:{extremum}?[^{punctuation}{ctrl}{invalid}{space}\d]({middle}{{1,60}}{extremum}|{extremum})?)",
        extremum=*VALID_DOMAIN_EXTREMA_CHARS,
        middle=*VALID_DOMAIN_MIDDLE_CHARS,
        punctuation = PUNCTUATION,
        ctrl = CTRL_CHARS,
        invalid = INVALID_CHARS,
        space = UNICODE_SPACES
    );

    /// Matches domains.
    static ref VALID_DOMAIN: String = format!(r"(?:(?:{part}\.)*{part}\.{tld})",
        part=*VALID_DOMAIN_SEGMENT, tld=*SYNTACTICALLY_VALID_TLD);

    /// Matches characters valid in a path segment.
    static ref VALID_PATH_CHARS: String = format!(r"[^{space}\(\)\?]", space=UNICODE_SPACES);
    /// Matches characters valid at the end of a path segment.
    static ref VALID_PATH_ENDING_CHARS: String = format!(
        r"[^{space}\(\)\?!\*';:=,\.\$%\[\]~&\|@]|(?:{balanced_parens})",
        space=UNICODE_SPACES,
        balanced_parens=*VALID_PATH_BALANCED_PARENS,
    );

    static ref VALID_PATH_BALANCED_PARENS: String = format!(concat!(
        r"\(",
            "(?:",
                "{path_char}+",
            "|",
                r"(?:\({path_char}+\))",
            ")",
        r"\)"),
        path_char=*VALID_PATH_CHARS,
    );

    /// Matches a valid segment of a path.
    static ref VALID_PATH_SEGMENT: String = format!(concat!(
        "(?:",
            "(?:{path_char}*(?:{balanced_parens}{path_char}*)*{path_ending_char})",
        "|",
            "(?:{path_char}+/)",
        ")"),
        path_char=*VALID_PATH_CHARS,
        path_ending_char=*VALID_PATH_ENDING_CHARS,
        balanced_parens=*VALID_PATH_BALANCED_PARENS
    );

    /// Matches a URL.
    static ref VALID_URL: String = format!(concat!(
        "(",                                     // $1 - whole match
            "(https?://)",                       // $2 - scheme
            "({domain})",                        // $3 - domain
            "(?::([0-9]+))?",                    // $4 - port
            "(/{path}*)?",                       // $5 - path
           r"(\?{query})?",                      // $6 - query
        ")"),
        domain = *VALID_DOMAIN,
        path = *VALID_PATH_SEGMENT,
        query = VALID_QUERY_STRING
    );

    /// Matches a URL.
    pub static ref RE_URL: Regex = RegexBuilder::new(&*VALID_URL)
        .case_insensitive(true)
        .build()
        .unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_domain() {
        let re = Regex::new(&format!("^{}$", *VALID_DOMAIN)).unwrap();
        assert!(re.is_match("activitypub.rocks"));
        assert!(re.is_match("rustodon.glitch.social"));

        assert!(!re.is_match(".github.com"));
        assert!(!re.is_match("github.com."));
        assert!(!re.is_match("github..com"));
        assert!(!re.is_match("github"));
    }

    #[test]
    fn parses_domain_part() {
        let re = Regex::new(&format!("^{}$", *VALID_DOMAIN_SEGMENT)).unwrap();
        assert!(re.is_match("github"));
        assert!(re.is_match("destroy-capitalism"));

        assert!(!re.is_match("-oops"));
        assert!(!re.is_match("oops-"));
    }

    #[test]
    fn parses_tld() {
        let re = Regex::new(&format!("^{}$", &*SYNTACTICALLY_VALID_TLD)).unwrap();
        assert!(re.is_match("com"));
        assert!(re.is_match("net"));
        assert!(re.is_match("fr"));
        assert!(re.is_match("space"));
        assert!(re.is_match("한국"));
        assert!(re.is_match("한"));
        assert!(re.is_match("x"));
        assert!(re.is_match("a-b"));
        assert!(re.is_match("c3"));
        assert!(re.is_match("4xn4--4ff-----f4"));
        assert!(re.is_match("xn--mgberp4a5d4ar"));
        assert!(re.is_match("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));

        assert!(!re.is_match("-"));
        assert!(!re.is_match("_"));
        assert!(!re.is_match("-a"));
        assert!(!re.is_match("a-"));
        assert!(!re.is_match("33"));
        assert!(!re.is_match("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
    }

    #[test]
    fn re_url_builds() {
        use lazy_static::initialize;
        initialize(&RE_URL);
    }
}
