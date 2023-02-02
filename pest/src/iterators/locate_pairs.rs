// pest. The Elegant Parser
// Copyright (c) 2018 Dragoș Tiselice
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use alloc::rc::Rc;
use alloc::vec::Vec;

use super::line_index::LineIndex;
use super::pair::{self, Pair};
use super::queueable_token::QueueableToken;
use super::{flat_pairs, FlatPairs};
use crate::RuleType;

/// An iterator over [`Pair`]s. It is created by [`Pairs::locatable`].
/// This will prepare locate of the (line, col) for each pair.
///
/// [`Pair`]: struct.Pair.html
/// [`Pairs::locatable`]: struct.Pairs.html#method.locatable
pub struct LocatablePairs<'i, R> {
    /// # Safety
    ///
    /// All `QueueableToken`s' `input_pos` must be valid character boundary indices into `input`.
    queue: Rc<Vec<QueueableToken<R>>>,
    input: &'i str,
    start: usize,
    end: usize,
    line_index: Rc<LineIndex>,
}

/// # Safety
///
/// All `QueueableToken`s' `input_pos` must be valid character boundary indices into `input`.
pub unsafe fn new<R: RuleType>(
    queue: Rc<Vec<QueueableToken<R>>>,
    input: &str,
    start: usize,
    end: usize,
) -> LocatablePairs<'_, R> {
    LocatablePairs {
        queue,
        input,
        line_index: Rc::new(LineIndex::new(input)),
        start,
        end,
    }
}

impl<'i, R: RuleType> LocatablePairs<'i, R> {
    /// Peek at the first inner `Pair` without changing the position of this iterator.
    #[inline]
    pub fn peek(&self) -> Option<Pair<'i, R>> {
        if self.start < self.end {
            Some(unsafe {
                pair::new(
                    Rc::clone(&self.queue),
                    self.input,
                    Some(Rc::clone(&self.line_index)),
                    self.start,
                )
            })
        } else {
            None
        }
    }

    fn pair(&self) -> usize {
        match self.queue[self.start] {
            QueueableToken::Start {
                end_token_index, ..
            } => end_token_index,
            _ => unreachable!(),
        }
    }

    fn pair_from_end(&self) -> usize {
        match self.queue[self.end - 1] {
            QueueableToken::End {
                start_token_index, ..
            } => start_token_index,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn flatten(self) -> FlatPairs<'i, R> {
        let mut pairs = unsafe { flat_pairs::new(self.queue, self.input, self.start, self.end) };
        pairs.line_index = Some(Rc::clone(&self.line_index));
        pairs
    }
}

impl<'i, R: RuleType> Iterator for LocatablePairs<'i, R> {
    type Item = Pair<'i, R>;

    fn next(&mut self) -> Option<Self::Item> {
        let pair = self.peek()?;

        self.start = self.pair() + 1;
        Some(pair)
    }
}

impl<'i, R: RuleType> DoubleEndedIterator for LocatablePairs<'i, R> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end <= self.start {
            return None;
        }

        self.end = self.pair_from_end();

        let pair = unsafe {
            pair::new(
                Rc::clone(&self.queue),
                self.input,
                Some(Rc::clone(&self.line_index)),
                self.end,
            )
        };

        Some(pair)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        macros::tests::{AbcParser, Rule},
        Parser,
    };

    #[test]
    fn test_locatable_line_col() {
        let mut pairs = AbcParser::parse(Rule::a, "abc\nefgh").unwrap().locatable();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "abc");
        assert_eq!(pair.line_col(), (1, 1));

        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "e");
        assert_eq!(pair.line_col(), (2, 1));

        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "fgh");
        assert_eq!(pair.line_col(), (2, 2));
    }

    #[test]
    fn test_locatable_flatten_line_col() {
        let mut pairs = AbcParser::parse(Rule::a, "abc\nefgh")
            .unwrap()
            .locatable()
            .flatten();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "abc");
        assert_eq!(pair.line_col(), (1, 1));

        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "b");
        assert_eq!(pair.line_col(), (1, 2));

        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "e");
        assert_eq!(pair.line_col(), (2, 1));

        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "fgh");
        assert_eq!(pair.line_col(), (2, 2));
    }

    #[test]
    fn test_locatable_rev_iter_line_col() {
        let mut pairs = AbcParser::parse(Rule::a, "abc\nefgh")
            .unwrap()
            .locatable()
            .rev();
        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "fgh");
        assert_eq!(pair.line_col(), (2, 2));

        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "e");
        assert_eq!(pair.line_col(), (2, 1));

        let pair = pairs.next().unwrap();
        assert_eq!(pair.as_str(), "abc");
        assert_eq!(pair.line_col(), (1, 1));
    }
}
