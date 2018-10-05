#[allow(unused_imports)]
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar;
