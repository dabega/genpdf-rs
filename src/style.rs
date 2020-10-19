// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Types for styled strings.
//!
//! A [`StyledString`][] is a [`String`][] with a [`Style`][] annotation.  Accordingly, a
//! [`StyledStr`][] is a [`&str`][] with a [`Style`][] annotation, and a [`StyledCow`][] is either
//! a [`Cow<'_, str>`][] with a [`Style`][] annotation.
//!
//! A [`Style`][] is a combination of a [`FontFamily`][], a font size, a line spacing factor, a
//! [`Color`][] and a combination of [`Effect`][]s (bold or italic).
//!
//! # Example
//!
//! ```
//! use genpdf::style;
//! let style = style::Style::new().bold();
//! let ss1 = style::StyledStr::new("bold", style);
//! let ss2 = style::StyledStr::new("red", style::Color::Rgb(255, 0, 0));
//! ```
//!
//! [`Color`]: enum.Color.html
//! [`Effect`]: enum.Effect.html
//! [`FontFamily`]: ../fonts/struct.FontFamily.html
//! [`Style`]: struct.Style.html
//! [`StyledCow`]: struct.StyledCow.html
//! [`StyledStr`]: struct.StyledStr.html
//! [`StyledString`]: struct.StyledString.html
//! [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
//! [`&str`]: https://doc.rust-lang.org/std/primitive.str.html
//! [`Cow<'_, str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html

use std::borrow;
use std::iter;

use crate::fonts;
use crate::Mm;

/// A color, represented by RGB, CMYK or Greyscale values.
///
/// For all variants, the possible values range from 0 to 255.
///
/// # Examples
///
/// ```
/// let red = genpdf::style::Color::Rgb(255, 0, 0);
/// let cyan = genpdf::style::Color::Cmyk(255, 0, 0, 0);
/// let grey = genpdf::style::Color::Greyscale(127);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// An RGB color with red, green and blue values between 0 and 255.
    Rgb(u8, u8, u8),
    /// An CMYK color with cyan, magenta, yellow and key values between 0 and 255.
    Cmyk(u8, u8, u8, u8),
    /// A greyscale color with a value between 0 and 255.
    Greyscale(u8),
}

impl From<Color> for printpdf::Color {
    fn from(color: Color) -> printpdf::Color {
        match color {
            Color::Rgb(r, g, b) => printpdf::Color::Rgb(printpdf::Rgb::new(
                f64::from(r) / 255.0,
                f64::from(g) / 255.0,
                f64::from(b) / 255.0,
                None,
            )),
            Color::Cmyk(c, m, y, k) => printpdf::Color::Cmyk(printpdf::Cmyk::new(
                f64::from(c) / 255.0,
                f64::from(m) / 255.0,
                f64::from(y) / 255.0,
                f64::from(k) / 255.0,
                None,
            )),
            Color::Greyscale(val) => {
                printpdf::Color::Greyscale(printpdf::Greyscale::new(f64::from(val) / 255.0, None))
            }
        }
    }
}

/// A text effect (bold or italic).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    /// Bold text.
    Bold,
    /// Italic text.
    Italic,
}

/// A style annotation for a string.
///
/// The annotation consists of:
/// - a font family, see [`FontFamily`][] (defaults to the [`FontCache`][] default)
/// - a font size in points (defaults to 12)
/// - a line spacing factor, with 1 meaning single line spacing (defaults to 1)
/// - an outline color, see [`Color`][] (defaults to black)
/// - a combination of text effects, see [`Effect`][] (defaults to none)
///
/// All properties are optional.  If they are not set, they can be inferred from parent styles or
/// from the defaults.
///
/// [`Color`]: enum.Color.html
/// [`Effect`]: enum.Effect.html
/// [`FontFamily`]: ../fonts/struct.FontFamily.html
/// [`FontCache`]: ../fonts/struct.FontCache.html
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Style {
    font_family: Option<fonts::FontFamily<fonts::Font>>,
    font_size: Option<u8>,
    line_spacing: Option<f64>,
    color: Option<Color>,
    is_bold: bool,
    is_italic: bool,
}

impl Style {
    /// Creates a new style without settings.
    pub fn new() -> Style {
        Style::default()
    }

    /// Merges the given style into this style.
    pub fn merge(&mut self, style: impl Into<Style>) {
        let style = style.into();
        if let Some(font_family) = style.font_family {
            self.font_family = Some(font_family);
        };
        if let Some(font_size) = style.font_size {
            self.font_size = Some(font_size);
        }
        if let Some(color) = style.color {
            self.color = Some(color);
        }
        if style.is_bold {
            self.is_bold = true;
        }
        if style.is_italic {
            self.is_italic = true;
        }
    }

    /// Combines this style and the given style and returns the result.
    pub fn and(mut self, style: impl Into<Style>) -> Style {
        self.merge(style);
        self
    }

    /// Creates a new style by combining the given styles.
    pub fn combine(left: impl Into<Style>, right: impl Into<Style>) -> Style {
        left.into().and(right)
    }

    /// Returns the outline color for this style, if set.
    pub fn color(&self) -> Option<Color> {
        self.color
    }

    /// Returns whether the bold text effect is set.
    pub fn is_bold(&self) -> bool {
        self.is_bold
    }

    /// Returns whether the italic text effect is set.
    pub fn is_italic(&self) -> bool {
        self.is_italic
    }

    /// Returns the font size for this style in points, or 12 if no font size is set.
    pub fn font_size(&self) -> u8 {
        self.font_size.unwrap_or(12)
    }

    /// Returns the line spacing factor for this style, or 1 if no line spacing factor is set.
    pub fn line_spacing(&self) -> f64 {
        self.line_spacing.unwrap_or(1.0)
    }

    /// Sets the bold effect for this style.
    pub fn set_bold(&mut self) {
        self.is_bold = true;
    }

    /// Sets the bold effect for this style and returns it.
    pub fn bold(mut self) -> Style {
        self.set_bold();
        self
    }

    /// Sets the italic effect for this style.
    pub fn set_italic(&mut self) {
        self.is_italic = true;
    }

    /// Sets the italic effect for this style and returns it.
    pub fn italic(mut self) -> Style {
        self.set_italic();
        self
    }

    /// Sets the font family for this style.
    pub fn set_font_family(&mut self, font_family: fonts::FontFamily<fonts::Font>) {
        self.font_family = Some(font_family);
    }

    /// Sets the font family for this style and returns it.
    pub fn with_font_family(mut self, font_family: fonts::FontFamily<fonts::Font>) -> Style {
        self.set_font_family(font_family);
        self
    }

    /// Sets the line spacing factor for this style.
    pub fn set_line_spacing(&mut self, line_spacing: f64) {
        self.line_spacing = Some(line_spacing);
    }

    /// Sets the line spacing factor for this style and returns it.
    pub fn with_line_spacing(mut self, line_spacing: f64) -> Style {
        self.set_line_spacing(line_spacing);
        self
    }

    /// Sets the font size in points for this style.
    pub fn set_font_size(&mut self, font_size: u8) {
        self.font_size = Some(font_size);
    }

    /// Sets the font size in points for this style and returns it.
    pub fn with_font_size(mut self, font_size: u8) -> Style {
        self.set_font_size(font_size);
        self
    }

    /// Sets the outline color for this style.
    pub fn set_color(&mut self, color: Color) {
        self.color = Some(color);
    }

    /// Sets the outline color for this style and returns it.
    pub fn with_color(mut self, color: Color) -> Self {
        self.set_color(color);
        self
    }

    /// Calculates the width of the given character with this style using the data in the given
    /// font cache.
    ///
    /// If the font family is set, it must have been created by the given [`FontCache`][].
    ///
    /// [`FontCache`]: ../fonts/struct.FontCache.html
    pub fn char_width(&self, font_cache: &fonts::FontCache, c: char) -> Mm {
        self.font(font_cache)
            .char_width(font_cache, c, self.font_size())
    }

    /// Calculates the width of the given string with this style using the data in the given font
    /// cache.
    ///
    /// If the font family is set, it must have been created by the given [`FontCache`][].
    ///
    /// [`FontCache`]: ../fonts/struct.FontCache.html
    pub fn str_width(&self, font_cache: &fonts::FontCache, s: &str) -> Mm {
        let font = self.font(font_cache);
        font.str_width(font_cache, s, self.font_size())
    }

    /// Returns the font family for this style or the default font family using the given font
    /// cache.
    ///
    /// If the font family is set, it must have been created by the given [`FontCache`][].
    ///
    /// [`FontCache`]: ../fonts/struct.FontCache.html
    pub fn font_family(&self, font_cache: &fonts::FontCache) -> fonts::FontFamily<fonts::Font> {
        self.font_family
            .unwrap_or_else(|| font_cache.default_font_family())
    }

    /// Returns the font for this style using the given font cache.
    ///
    /// If the font family is set, it must have been created by the given [`FontCache`][].
    ///
    /// [`FontCache`]: ../fonts/struct.FontCache.html
    pub fn font(&self, font_cache: &fonts::FontCache) -> fonts::Font {
        self.font_family(font_cache).get(*self)
    }

    /// Calculates the line height for strings with this style using the data in the given font
    /// cache.
    ///
    /// If the font family is set, it must have been created by the given [`FontCache`][].
    ///
    /// [`FontCache`]: ../fonts/struct.FontCache.html
    pub fn line_height(&self, font_cache: &fonts::FontCache) -> Mm {
        self.font(font_cache).get_line_height(self.font_size()) * self.line_spacing()
    }
}

impl From<Color> for Style {
    fn from(color: Color) -> Style {
        Style::new().with_color(color)
    }
}

impl From<Effect> for Style {
    fn from(effect: Effect) -> Style {
        let style = Style::new();
        match effect {
            Effect::Bold => style.bold(),
            Effect::Italic => style.italic(),
        }
    }
}

impl From<fonts::FontFamily<fonts::Font>> for Style {
    fn from(font_family: fonts::FontFamily<fonts::Font>) -> Style {
        Style::new().with_font_family(font_family)
    }
}

impl<T: Into<Style>> iter::Extend<T> for Style {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for style in iter {
            self.merge(style.into());
        }
    }
}

impl<T: Into<Style>> iter::FromIterator<T> for Style {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Style {
        let mut style = Style::new();
        style.extend(iter);
        style
    }
}

/// A [`String`][] with a [`Style`][] annotation.
///
/// # Example
///
/// ```
/// use genpdf::style;
/// let ss1 = style::StyledString::new("bold".to_owned(), style::Effect::Bold);
/// let ss2 = style::StyledString::new("red".to_owned(), style::Color::Rgb(255, 0, 0));
/// ```
///
/// [`Style`]: struct.Style.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
#[derive(Clone, Debug, Default)]
pub struct StyledString {
    /// The annotated string.
    pub s: String,
    /// The style annotation.
    pub style: Style,
}

impl StyledString {
    /// Creates a new styled string from the given string and style.
    pub fn new(s: impl Into<String>, style: impl Into<Style>) -> StyledString {
        StyledString {
            s: s.into(),
            style: style.into(),
        }
    }

    /// Calculates the width of the this string with this style using the data in the given font
    /// cache.
    ///
    /// If the font family is set for the style, it must have been created by the given
    /// [`FontCache`][].
    ///
    /// [`FontCache`]: ../fonts/struct.FontCache.html
    pub fn width(&self, font_cache: &fonts::FontCache) -> Mm {
        self.style.str_width(font_cache, &self.s)
    }
}

impl From<String> for StyledString {
    fn from(s: String) -> StyledString {
        StyledString::new(s, Style::new())
    }
}

impl<'a> From<&'a String> for StyledString {
    fn from(s: &'a String) -> StyledString {
        StyledString::from(s.to_owned())
    }
}

impl<'a> From<&'a str> for StyledString {
    fn from(s: &'a str) -> StyledString {
        StyledString::from(s.to_owned())
    }
}

/// A [`&str`][] with a [`Style`][] annotation.
///
/// # Example
///
/// ```
/// use genpdf::style;
/// let ss1 = style::StyledStr::new("bold", style::Effect::Bold);
/// let ss2 = style::StyledStr::new("red", style::Color::Rgb(255, 0, 0));
/// ```
///
/// [`Style`]: struct.Style.html
/// [`&str`]: https://doc.rust-lang.org/std/primitive.str.html
#[derive(Clone, Copy, Debug, Default)]
pub struct StyledStr<'s> {
    /// The annotated string.
    pub s: &'s str,
    /// The style annotation.
    pub style: Style,
}

impl<'s> StyledStr<'s> {
    /// Creates a new styled string from the given string and style.
    pub fn new(s: &'s str, style: impl Into<Style>) -> StyledStr<'s> {
        StyledStr {
            s,
            style: style.into(),
        }
    }

    /// Calculates the width of the this string with this style using the data in the given font
    /// cache.
    ///
    /// If the font family is set for the style, it must have been created by the given
    /// [`FontCache`][].
    ///
    /// [`FontCache`]: ../fonts/struct.FontCache.html
    pub fn width(&self, font_cache: &fonts::FontCache) -> Mm {
        self.style.str_width(font_cache, &self.s)
    }
}

impl<'s> From<&'s str> for StyledStr<'s> {
    fn from(s: &'s str) -> StyledStr<'s> {
        StyledStr::new(s, Style::new())
    }
}

impl<'s> From<&'s String> for StyledStr<'s> {
    fn from(s: &'s String) -> StyledStr<'s> {
        StyledStr::new(s, Style::new())
    }
}

impl<'s> From<&'s StyledString> for StyledStr<'s> {
    fn from(s: &'s StyledString) -> StyledStr<'s> {
        StyledStr::new(&s.s, s.style)
    }
}

/// A [`Cow<'s, str>`][] with a [`Style`][] annotation.
///
/// # Example
///
/// ```
/// use genpdf::style;
/// let ss1 = style::StyledCow::new("bold", style::Effect::Bold);
/// let ss2 = style::StyledCow::new("red".to_owned(), style::Color::Rgb(255, 0, 0));
/// ```
///
/// [`Style`]: struct.Style.html
/// [`Cow<'s, str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
#[derive(Clone, Debug, Default)]
pub struct StyledCow<'s> {
    /// The annotated string.
    pub s: borrow::Cow<'s, str>,
    /// The style annotation.
    pub style: Style,
}

impl<'s> StyledCow<'s> {
    /// Creates a new styled string from the given string and style.
    pub fn new(s: impl Into<borrow::Cow<'s, str>>, style: impl Into<Style>) -> StyledCow<'s> {
        StyledCow {
            s: s.into(),
            style: style.into(),
        }
    }

    /// Calculates the width of the this string with this style using the data in the given font
    /// cache.
    ///
    /// If the font family is set for the style, it must have been created by the given
    /// [`FontCache`][].
    ///
    /// [`FontCache`]: ../fonts/struct.FontCache.html
    pub fn width(&self, font_cache: &fonts::FontCache) -> Mm {
        self.style.str_width(font_cache, self.s.as_ref())
    }
}

impl<'s> From<&'s str> for StyledCow<'s> {
    fn from(s: &'s str) -> StyledCow<'s> {
        StyledCow::new(s, Style::new())
    }
}

impl<'s> From<&'s String> for StyledCow<'s> {
    fn from(s: &'s String) -> StyledCow<'s> {
        StyledCow::new(s, Style::new())
    }
}

impl<'s> From<String> for StyledCow<'s> {
    fn from(s: String) -> StyledCow<'s> {
        StyledCow::new(s, Style::new())
    }
}

impl<'s> From<StyledStr<'s>> for StyledCow<'s> {
    fn from(s: StyledStr<'s>) -> StyledCow<'s> {
        StyledCow::new(s.s, s.style)
    }
}

impl<'s> From<&'s StyledString> for StyledCow<'s> {
    fn from(s: &'s StyledString) -> StyledCow<'s> {
        StyledCow::new(&s.s, s.style)
    }
}

impl<'s> From<StyledString> for StyledCow<'s> {
    fn from(s: StyledString) -> StyledCow<'s> {
        StyledCow::new(s.s, s.style)
    }
}
