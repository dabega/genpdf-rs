// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

///! Utilities for text wrapping.
use crate::fonts;
use crate::style;
use crate::Mm;

pub struct Wrapper<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> {
    iter: I,
    font_cache: &'c fonts::FontCache,
    width: Mm,
    x: Mm,
    buf: Vec<style::StyledStr<'s>>,
}

impl<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> Wrapper<'c, 's, I> {
    pub fn new(iter: I, font_cache: &'c fonts::FontCache, width: Mm) -> Wrapper<'c, 's, I> {
        Wrapper {
            iter,
            font_cache,
            width,
            x: Mm(0.0),
            buf: Vec::new(),
        }
    }
}

impl<'c, 's, I: Iterator<Item = style::StyledStr<'s>>> Iterator for Wrapper<'c, 's, I> {
    type Item = Vec<style::StyledStr<'s>>;

    fn next(&mut self) -> Option<Vec<style::StyledStr<'s>>> {
        while let Some(s) = self.iter.next() {
            let width = s.width(self.font_cache);
            if width > self.width {
                // TODO: handle
                break;
            } else if self.x + width > self.width {
                let v = std::mem::take(&mut self.buf);
                self.buf.push(s);
                self.x = width;
                return Some(v);
            } else {
                self.buf.push(s);
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

pub struct Words<'s, I: Iterator<Item = style::StyledStr<'s>>> {
    iter: I,
    offset: usize,
    s: Option<style::StyledStr<'s>>,
}

impl<'s, I: Iterator<Item = style::StyledStr<'s>>> Words<'s, I> {
    pub fn new<IntoIter: IntoIterator<Item = style::StyledStr<'s>, IntoIter = I>>(
        iter: IntoIter,
        offset: usize,
    ) -> Words<'s, I> {
        Words {
            iter: iter.into_iter(),
            offset,
            s: None,
        }
    }
}

impl<'s, I: Iterator<Item = style::StyledStr<'s>>> Iterator for Words<'s, I> {
    type Item = style::StyledStr<'s>;

    fn next(&mut self) -> Option<style::StyledStr<'s>> {
        if self.s.as_ref().map(|s| s.s.is_empty()).unwrap_or(true) {
            self.s = self.iter.next();
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
