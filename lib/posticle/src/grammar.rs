pub use pest::error::Error;
use pest::iterators::Pairs;
#[allow(unused_imports)]
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct Grammar;

/// Parse an entire document of posticle style markup.
pub fn document(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    Grammar::parse(Rule::document, input)
}

/// Parse a single emoticon.
pub fn emoticon(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    Grammar::parse(Rule::emoticon, input)
}

/// Parse a single hashtag.
pub fn hashtag(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    Grammar::parse(Rule::hashtag, input)
}

/// Parse a single link.
pub fn link(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    Grammar::parse(Rule::link, input)
}

/// Parse a single mention.
pub fn mention(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    Grammar::parse(Rule::mention, input)
}
