// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Utilities for text wrapping.

use crate::style;
use crate::Context;
use crate::Mm;

pub struct Wrapper<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> {
    iter: I,
    context: &'c Context,
    width: Mm,
    x: Mm,
    buf: Vec<style::StyledCow<'s>>,
}

impl<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> Wrapper<'c, 's, I> {
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
        while let Some(s) = self.iter.next() {
            let mut width = s.width(&self.context.font_cache);
            if self.x + width > self.width {
                let s = if let Some((start, end)) = split(self.context, s, self.width - self.x) {
                    self.buf.push(start);
                    width = s.width(&self.context.font_cache);
                    end
                } else {
                    s.into()
                };

                if width > self.width {
                    // TODO: handle
                    break;
                }
                let v = std::mem::take(&mut self.buf);
                self.buf.push(s);
                self.x = width;
                return Some(v);
            } else {
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

#[cfg(feature = "hyphenation")]
fn split<'s>(
    context: &Context,
    s: style::StyledStr<'s>,
    len: Mm,
) -> Option<(style::StyledCow<'s>, style::StyledCow<'s>)> {
    use hyphenation::{Hyphenator, Iter};

    let hyphenator = if let Some(hyphenator) = &context.hyphenator {
        hyphenator
    } else {
        return None;
    };

    let mark = "-";
    let mark_len = s.style.str_width(&context.font_cache, mark);

    let hyphenated = hyphenator.hyphenate(s.s);
    let segments: Vec<_> = hyphenated.iter().segments().collect();
    let idx = segments
        .iter()
        .scan(Mm(0.0), |acc, t| {
            *acc += s.style.str_width(&context.font_cache, t);
            Some(*acc)
        })
        .position(|w| w + mark_len > len)
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

pub struct Words<'s, S: Into<style::StyledStr<'s>>, I: Iterator<Item = S>> {
    iter: I,
    offset: usize,
    s: Option<style::StyledStr<'s>>,
}

impl<'s, S: Into<style::StyledStr<'s>>, I: Iterator<Item = S>> Words<'s, S, I> {
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

            let n = s.s.find(' ').map(|i| i + 1).unwrap_or_else(|| s.s.len());
            let (word, rest) = s.s.split_at(n);
            s.s = rest;
            Some(style::StyledStr::new(word, s.style))
        } else {
            None
        }
    }
}
