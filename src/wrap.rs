// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Utilities for text wrapping.

use crate::style;
use crate::Context;
use crate::Mm;

/// Combines a sequence of styled words into lines with a maximum width.
///
/// If a word does not fit into a line, the wrapper tries to split it using the `split` function.
pub struct Wrapper<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> {
    iter: I,
    context: &'c Context,
    width: Mm,
    x: Mm,
    buf: Vec<style::StyledCow<'s>>,
}

impl<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> Wrapper<'c, 's, I> {
    /// Creates a new wrapper for the given word sequence and with the given maximum width.
    pub fn new(iter: I, context: &'c Context, width: Mm) -> Wrapper<'c, 's, I> {
        Wrapper {
            iter,
            context,
            width,
            x: Mm(0.0),
            buf: Vec::new(),
        }
    }
}

impl<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> Iterator for Wrapper<'c, 's, I> {
    type Item = Vec<style::StyledCow<'s>>;

    fn next(&mut self) -> Option<Vec<style::StyledCow<'s>>> {
        // Append words to self.buf until the maximum line length is reached
        while let Some(s) = self.iter.next() {
            let mut width = s.width(&self.context.font_cache);

            if self.x + width > self.width {
                // The word does not fit into the current line (at least not completely)

                // Try to split the word so that the first part fits into the current line
                let s = if let Some((start, end)) = split(self.context, s, self.width - self.x) {
                    self.buf.push(start);
                    width = s.width(&self.context.font_cache);
                    end
                } else {
                    s.into()
                };

                if width > self.width {
                    // The remainder of the word is longer than the current page – we will never be
                    // able to render it completely.
                    // TODO: return error?
                    break;
                }

                // Return the current line and add the word that did not fit to the next line
                let v = std::mem::take(&mut self.buf);
                self.buf.push(s);
                self.x = width;
                return Some(v);
            } else {
                // The word fits in the current line, so just append it
                self.buf.push(s.into());
                self.x += width;
            }
        }

        if self.buf.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.buf))
        }
    }
}

#[cfg(not(feature = "hyphenation"))]
fn split<'s>(
    _context: &Context,
    _s: style::StyledStr<'s>,
    _len: Mm,
) -> Option<(style::StyledCow<'s>, style::StyledCow<'s>)> {
    None
}

/// Tries to split the given string into two parts so that the first part is shorter than the given
/// width.
#[cfg(feature = "hyphenation")]
fn split<'s>(
    context: &Context,
    s: style::StyledStr<'s>,
    width: Mm,
) -> Option<(style::StyledCow<'s>, style::StyledCow<'s>)> {
    use hyphenation::{Hyphenator, Iter};

    let hyphenator = if let Some(hyphenator) = &context.hyphenator {
        hyphenator
    } else {
        return None;
    };

    let mark = "-";
    let mark_width = s.style.str_width(&context.font_cache, mark);

    let hyphenated = hyphenator.hyphenate(s.s);
    let segments: Vec<_> = hyphenated.iter().segments().collect();

    // Find the hyphenation with the longest first part so that the first part (and the hyphen) are
    // shorter than or equals to the required width.
    let idx = segments
        .iter()
        .scan(Mm(0.0), |acc, t| {
            *acc += s.style.str_width(&context.font_cache, t);
            Some(*acc)
        })
        .position(|w| w + mark_width > width)
        .unwrap_or_default();
    if idx > 0 {
        let idx = hyphenated.breaks[idx - 1];
        let start = s.s[..idx].to_owned() + mark;
        let end = &s.s[idx..];
        Some((
            style::StyledCow::new(start, s.style),
            style::StyledCow::new(end, s.style),
        ))
    } else {
        None
    }
}

/// Splits a sequence of styled strings into words.
pub struct Words<'s, S: Into<style::StyledStr<'s>>, I: Iterator<Item = S>> {
    iter: I,
    offset: usize,
    s: Option<style::StyledStr<'s>>,
}

impl<'s, S: Into<style::StyledStr<'s>>, I: Iterator<Item = S>> Words<'s, S, I> {
    /// Creates a new words iterator.
    ///
    /// If `offset` is larger than zero, it determines the number of bytes to skip in the first
    /// string returned by the given iterator.
    pub fn new<IntoIter: IntoIterator<Item = S, IntoIter = I>>(
        iter: IntoIter,
        offset: usize,
    ) -> Words<'s, S, I> {
        Words {
            iter: iter.into_iter(),
            offset,
            s: None,
        }
    }
}

impl<'s, S: Into<style::StyledStr<'s>>, I: Iterator<Item = S>> Iterator for Words<'s, S, I> {
    type Item = style::StyledStr<'s>;

    fn next(&mut self) -> Option<style::StyledStr<'s>> {
        if self.s.as_ref().map(|s| s.s.is_empty()).unwrap_or(true) {
            self.s = self.iter.next().map(Into::into);
        }

        if let Some(s) = &mut self.s {
            if self.offset > 0 {
                s.s = &s.s[self.offset..];
                self.offset = 0;
            }

            // Split at the first space or use the complete string
            let n = s.s.find(' ').map(|i| i + 1).unwrap_or_else(|| s.s.len());
            let (word, rest) = s.s.split_at(n);
            s.s = rest;
            Some(style::StyledStr::new(word, s.style))
        } else {
            None
        }
    }
}
