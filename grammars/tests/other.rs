#[macro_use]
extern crate pest;
extern crate pest_grammars;

use pest::parses_to;
use pest_grammars::other::OtherParser;

use pest_grammars::other::*;

#[test]
fn test_rep_min_max() {
    parses_to! {
        parser: OtherParser,
        input: "1234A",
        rule: Rule::rep_min_max,
        tokens: [
            rep_min_max(0, 4)
        ]
    };

    parses_to! {
        parser: OtherParser,
        input: "1234A",
        rule: Rule::rep_min_max_large,
        tokens: [
            rep_min_max_large(0, 4)
        ]
    };
}
