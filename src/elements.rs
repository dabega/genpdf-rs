// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Elements of a PDF document.
//!
//! This module provides implementations of the [`Element`][] trait that can be used to render and
//! arrange text and shapes.
//!
//! It includes the following elements:
//! - Containers:
//!   - [`LinearLayout`][]: arranges its elements sequentially
//!   - [`TableLayout`][]: arranges its elements in columns and rows
//!   - [`OrderedList`][] and [`UnorderedList`][]: arrange their elements sequentially with bullet
//!     points
//! - Text:
//!   - [`Text`][]: a single line of text
//!   - [`Paragraph`][]: a wrapped and aligned paragraph of text
//! - Wrappers:
//!   - [`FramedElement`][]: draws a frame around the wrapped element
//!   - [`PaddedElement`][]: adds a padding to the wrapped element
//!   - [`StyledElement`][]: sets a default style for the wrapped element and its children
//! - Other:
//!   - [`Break`][]: adds forced line breaks as a spacer
//!
//! You can create custom elements by implementing the [`Element`][] trait.
//!
//! [`Element`]: ../trait.Element.html
//! [`LinearLayout`]: struct.LinearLayout.html
//! [`TableLayout`]: struct.TableLayout.html
//! [`OrderedList`]: struct.OrderedList.html
//! [`UnorderedList`]: struct.UnorderedList.html
//! [`Text`]: struct.Text.html
//! [`Break`]: struct.Break.html
//! [`Paragraph`]: struct.Paragraph.html
//! [`FramedElement`]: struct.FramedElement.html
//! [`PaddedElement`]: struct.PaddedElement.html
//! [`StyledElement`]: struct.StyledElement.html

use std::iter;

use crate::error::{Error, ErrorKind};
use crate::render;
use crate::style::{Style, StyledString};
use crate::wrap;
use crate::{Context, Element, Margins, Mm, Position, RenderResult, Size};

/// Arranges a list of elements sequentially.
///
/// Currently, elements can only be arranged vertically.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut layout = elements::LinearLayout::vertical();
/// layout.push(elements::Paragraph::new("Test1"));
/// layout.push(elements::Paragraph::new("Test2"));
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let layout = elements::LinearLayout::vertical()
///     .element(elements::Paragraph::new("Test1"))
///     .element(elements::Paragraph::new("Test2"));
/// ```
///
pub struct LinearLayout {
    elements: Vec<Box<dyn Element>>,
    render_idx: usize,
}

impl LinearLayout {
    fn new() -> LinearLayout {
        LinearLayout {
            elements: Vec::new(),
            render_idx: 0,
        }
    }

    /// Creates a new linear layout that arranges its elements vertically.
    pub fn vertical() -> LinearLayout {
        LinearLayout::new()
    }

    /// Adds the given element to this layout.
    pub fn push<E: Element + 'static>(&mut self, element: E) {
        self.elements.push(Box::new(element));
    }

    /// Adds the given element to this layout and it returns the layout.
    pub fn element<E: Element + 'static>(mut self, element: E) -> Self {
        self.push(element);
        self
    }

    fn render_vertical(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        while area.size().height > Mm(0.0) && self.render_idx < self.elements.len() {
            let element_result =
                self.elements[self.render_idx].render(context, area.clone(), style)?;
            area.add_offset(Position::new(0, element_result.size.height));
            result.size = result.size.stack_vertical(element_result.size);
            if element_result.has_more {
                result.has_more = true;
                return Ok(result);
            }
            self.render_idx += 1;
        }
        result.has_more = self.render_idx < self.elements.len();
        Ok(result)
    }
}

impl Element for LinearLayout {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        // TODO: add horizontal layout
        self.render_vertical(context, area, style)
    }
}

/// A single line of formatted text.
///
/// This element renders a single styled string on a single line.  It does not wrap it if the
/// string is longer than the line.  Therefore you should prefer [`Paragraph`][] over `Text` for
/// most use cases.
///
/// [`Paragraph`]: struct.Paragraph.html
#[derive(Clone, Debug, Default)]
pub struct Text {
    text: StyledString,
}

impl Text {
    /// Creates a new instance with the given styled string.
    pub fn new(text: impl Into<StyledString>) -> Text {
        Text { text: text.into() }
    }
}

impl Element for Text {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        mut style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        style.merge(self.text.style);
        if area.print_str(
            &context.font_cache,
            Position::default(),
            style,
            &self.text.s,
        )? {
            result.size = Size::new(
                style.str_width(&context.font_cache, &self.text.s),
                style.line_height(&context.font_cache),
            );
        } else {
            result.has_more = true;
        }
        Ok(result)
    }
}

/// The alignment of a [`Paragraph`][].
///
/// The default alignment is left-flushed.
///
/// [`Paragraph`]: struct.Paragraph.html
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Alignment {
    /// Left-flushed.
    Left,
    /// Right-flushed.
    Right,
    /// Centered.
    Center,
}

impl Default for Alignment {
    fn default() -> Alignment {
        Alignment::Left
    }
}

/// A multi-line wrapped paragraph of formatted text.
///
/// If the text of this paragraph is longer than the page width, the paragraph is wrapped at word
/// borders (and additionally at string borders if it contains multiple strings).  If a word in the
/// paragraph is longer than the page width, the text is truncated.
///
/// Use the [`push`][], [`string`][], [`push_styled`][] and [`string_styled`][] methods to add
/// strings to this paragraph.  Besides the styling of the text (see [`Style`][]), you can also set
/// an [`Alignment`][] for the paragraph.
///
/// Note that the line height and spacing is currently calculated based on the style of the entire
/// paragraph.  If the font family or font size is changed in the [`Style`][] settings for a
/// string, the line height and spacing might be incorrect.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::{elements, style};
/// let mut p = elements::Paragraph::default();
/// p.push("This is an ");
/// p.push_styled("important", style::Color::Rgb(255, 0, 0));
/// p.push(" message!");
/// p.set_alignment(elements::Alignment::Center);
/// ```
///
/// Chained:
/// ```
/// use genpdf::{elements, style};
/// let p = elements::Paragraph::default()
///     .string("This is an ")
///     .styled_string("important", style::Color::Rgb(255, 0, 0))
///     .string(" message!")
///     .aligned(elements::Alignment::Center);
/// ```
///
/// [`Style`]: ../style/struct.Style.html
/// [`Alignment`]: enum.Alignment.html
/// [`push`]: #method.push
/// [`push_styled`]: #method.push_styled
/// [`string`]: #method.string
/// [`string_styled`]: #method.string_styled
#[derive(Clone, Debug, Default)]
pub struct Paragraph {
    text: Vec<StyledString>,
    render_offset: usize,
    render_idx: usize,
    style_applied: bool,
    alignment: Alignment,
}

impl Paragraph {
    /// Creates a new paragraph with the given content.
    pub fn new(text: impl Into<StyledString>) -> Paragraph {
        Paragraph {
            text: vec![text.into()],
            ..Default::default()
        }
    }

    /// Sets the alignment of this paragraph.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.alignment = alignment;
    }

    /// Sets the alignment of this paragraph and returns the paragraph.
    pub fn aligned(mut self, alignment: Alignment) -> Self {
        self.set_alignment(alignment);
        self
    }

    /// Adds a string to the end of this paragraph.
    pub fn push(&mut self, s: impl Into<StyledString>) {
        self.text.push(s.into());
    }

    /// Adds a string to the end of this paragraph and returns the paragraph.
    pub fn string(mut self, s: impl Into<StyledString>) -> Self {
        self.push(s);
        self
    }

    /// Adds a string with the given style to the end of this paragraph.
    pub fn push_styled(&mut self, s: impl Into<String>, style: impl Into<Style>) {
        self.text.push(StyledString::new(s, style))
    }

    /// Adds a string with the given style to the end of this paragraph and returns the paragraph.
    pub fn styled_string(mut self, s: impl Into<String>, style: impl Into<Style>) -> Self {
        self.push_styled(s, style);
        self
    }

    fn get_offset(&self, width: Mm, max_width: Mm) -> Mm {
        match self.alignment {
            Alignment::Left => Mm::default(),
            Alignment::Center => (max_width - width) / 2.0,
            Alignment::Right => max_width - width,
        }
    }

    fn apply_style(&mut self, style: Style) {
        if !self.style_applied {
            for s in &mut self.text {
                s.style = style.and(s.style);
            }
            self.style_applied = true;
        }
    }
}

impl Element for Paragraph {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        if self.render_idx >= self.text.len() {
            return Ok(result);
        }

        self.apply_style(style);

        let height = style.line_height(&context.font_cache);
        let words = wrap::Words::new(self.text.iter().skip(self.render_idx), self.render_offset);
        for line in wrap::Wrapper::new(words, context, area.size().width) {
            let width = line.iter().map(|s| s.width(&context.font_cache)).sum();
            let position = Position::new(self.get_offset(width, area.size().width), 0);
            // TODO: calculate the maximum line height
            if let Ok(mut section) = area.text_section(&context.font_cache, position, style) {
                for s in line {
                    section.print_str(&s.s, s.style)?;
                    self.render_offset += s.s.len();
                    while self.render_idx < self.text.len()
                        && self.render_offset >= self.text[self.render_idx].s.len()
                    {
                        self.render_offset -= self.text[self.render_idx].s.len();
                        self.render_idx += 1;
                    }
                }
            } else {
                result.has_more = true;
                break;
            }
            result.size = result.size.stack_vertical(Size::new(width, height));
            area.add_offset(Position::new(0, height));
        }

        Ok(result)
    }
}

impl<T: Into<StyledString>> iter::Extend<T> for Paragraph {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for s in iter {
            self.push(s);
        }
    }
}

impl<T: Into<StyledString>> iter::FromIterator<T> for Paragraph {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Paragraph {
        let mut paragraph = Paragraph::default();
        paragraph.extend(iter);
        paragraph
    }
}

/// A line break.
///
/// This element inserts a given number of empty lines.
///
/// # Example
///
/// ```
/// // Draws 5 empty lines (calculating the line height using the current style)
/// let b = genpdf::elements::Break::new(5);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Break {
    lines: f64,
}

impl Break {
    /// Creates a new break with the given number of lines.
    pub fn new(lines: impl Into<f64>) -> Break {
        Break {
            lines: lines.into(),
        }
    }
}

impl Element for Break {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        if self.lines <= 0.0 {
            return Ok(result);
        }
        let line_height = style.line_height(&context.font_cache);
        let break_height = line_height * self.lines;
        if break_height < area.size().height {
            result.size.height = break_height;
            self.lines = 0.0;
        } else {
            result.size.height = area.size().height;
            self.lines -= result.size.height.0 / line_height.0;
        }
        Ok(result)
    }
}

/// Adds a padding to the wrapped element.
///
/// # Examples
///
/// Direct usage:
/// ```
/// use genpdf::elements;
/// let p = elements::PaddedElement::new(
///     elements::Paragraph::new("text"),
///     genpdf::Margins::trbl(5, 2, 5, 10),
/// );
/// ```
///
/// Using [`Element::padded`][]:
/// ```
/// use genpdf::{elements, Element as _};
/// let p = elements::Paragraph::new("text")
///     .padded(genpdf::Margins::trbl(5, 2, 5, 10));
/// ```
///
/// [`Element::padded`]: ../trait.Element.html#method.padded
#[derive(Clone, Debug, Default)]
pub struct PaddedElement<E: Element> {
    element: E,
    padding: Margins,
}

impl<E: Element> PaddedElement<E> {
    /// Creates a new padded element that wraps the given element with the given padding.
    pub fn new(element: E, padding: impl Into<Margins>) -> PaddedElement<E> {
        PaddedElement {
            element,
            padding: padding.into(),
        }
    }
}

impl<E: Element> Element for PaddedElement<E> {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        area.add_margins(Margins {
            bottom: Mm(0.0),
            ..self.padding
        });
        let mut result = self.element.render(context, area, style)?;
        result.size.width += self.padding.left + self.padding.right;
        result.size.height += self.padding.top + self.padding.bottom;
        Ok(result)
    }
}

/// Adds a default style to the wrapped element and its children.
///
/// # Examples
///
/// Direct usage:
/// ```
/// use genpdf::{elements, style};
/// let p = elements::StyledElement::new(
///     elements::Paragraph::new("text"),
///     style::Effect::Bold,
/// );
/// ```
///
/// Using [`Element::styled`][]:
/// ```
/// use genpdf::{elements, style, Element as _};
/// let p = elements::Paragraph::new("text")
///     .styled(style::Effect::Bold);
/// ```
///
/// [`Element::styled`]: ../trait.Element.html#method.styled
#[derive(Clone, Debug, Default)]
pub struct StyledElement<E: Element> {
    element: E,
    style: Style,
}

impl<E: Element> StyledElement<E> {
    /// Creates a new styled element that wraps the given element with the given style.
    pub fn new(element: E, style: impl Into<Style>) -> StyledElement<E> {
        StyledElement {
            element,
            style: style.into(),
        }
    }
}

impl<E: Element> Element for StyledElement<E> {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        mut style: Style,
    ) -> Result<RenderResult, Error> {
        style.merge(self.style);
        self.element.render(context, area, style)
    }
}

/// Adds a frame around the wrapped element.
///
/// # Examples
///
/// Direct usage:
/// ```
/// use genpdf::elements;
/// let p = elements::FramedElement::new(
///     elements::Paragraph::new("text"),
/// );
/// ```
///
/// Using [`Element::framed`][]:
/// ```
/// use genpdf::{elements, Element as _};
/// let p = elements::Paragraph::new("text").framed();
/// ```
///
/// [`Element::framed`]: ../trait.Element.html#method.framed
#[derive(Clone, Debug, Default)]
pub struct FramedElement<E: Element> {
    element: E,
    is_first: bool,
}

impl<E: Element> FramedElement<E> {
    /// Creates a new framed element that wraps the given element.
    pub fn new(element: E) -> FramedElement<E> {
        FramedElement {
            element,
            is_first: true,
        }
    }
}

impl<E: Element> Element for FramedElement<E> {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let result = self.element.render(context, area.clone(), style)?;
        area.draw_line(
            vec![Position::default(), Position::new(0, result.size.height)],
            style,
        );
        area.draw_line(
            vec![
                Position::new(area.size().width, 0),
                Position::new(area.size().width, result.size.height),
            ],
            style,
        );
        if self.is_first {
            area.draw_line(
                vec![Position::default(), Position::new(area.size().width, 0)],
                style,
            );
        }
        if !result.has_more {
            area.draw_line(
                vec![
                    Position::new(0, result.size.height),
                    Position::new(area.size().width, result.size.height),
                ],
                style,
            );
        }
        self.is_first = false;
        Ok(result)
    }
}

/// An unordered list of elements with bullet points.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut list = elements::UnorderedList::new();
/// list.push(elements::Paragraph::new("first"));
/// list.push(elements::Paragraph::new("second"));
/// list.push(elements::Paragraph::new("third"));
/// ```
///
/// With setters and a custom bullet symbol:
/// ```
/// use genpdf::elements;
/// let mut list = elements::UnorderedList::with_bullet("*");
/// list.push(elements::Paragraph::new("first"));
/// list.push(elements::Paragraph::new("second"));
/// list.push(elements::Paragraph::new("third"));
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let list = elements::UnorderedList::new()
///     .element(elements::Paragraph::new("first"))
///     .element(elements::Paragraph::new("second"))
///     .element(elements::Paragraph::new("third"));
/// ```
pub struct UnorderedList {
    layout: LinearLayout,
    bullet: Option<String>,
}

impl UnorderedList {
    /// Creates a new unordered list with the default bullet point symbol.
    pub fn new() -> UnorderedList {
        UnorderedList {
            layout: LinearLayout::vertical(),
            bullet: None,
        }
    }

    /// Creates a new unordered list with the given bullet point symbol.
    pub fn with_bullet(bullet: impl Into<String>) -> UnorderedList {
        UnorderedList {
            layout: LinearLayout::vertical(),
            bullet: Some(bullet.into()),
        }
    }

    /// Adds an element to this list.
    pub fn push<E: Element + 'static>(&mut self, element: E) {
        let mut point = BulletPoint::new(element);
        if let Some(bullet) = &self.bullet {
            point.set_bullet(bullet.clone());
        }
        self.layout.push(point);
    }

    /// Adds an element to this list and returns the list.
    pub fn element<E: Element + 'static>(mut self, element: E) -> Self {
        self.push(element);
        self
    }
}

impl Element for UnorderedList {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        self.layout.render(context, area, style)
    }
}

impl Default for UnorderedList {
    fn default() -> UnorderedList {
        UnorderedList::new()
    }
}

/// An ordered list of elements with arabic numbers.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut list = elements::OrderedList::new();
/// list.push(elements::Paragraph::new("first"));
/// list.push(elements::Paragraph::new("second"));
/// list.push(elements::Paragraph::new("third"));
/// ```
///
/// With setters and a custom start number:
/// ```
/// use genpdf::elements;
/// let mut list = elements::OrderedList::with_start(5);
/// list.push(elements::Paragraph::new("first"));
/// list.push(elements::Paragraph::new("second"));
/// list.push(elements::Paragraph::new("third"));
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let list = elements::OrderedList::new()
///     .element(elements::Paragraph::new("first"))
///     .element(elements::Paragraph::new("second"))
///     .element(elements::Paragraph::new("third"));
/// ```
pub struct OrderedList {
    layout: LinearLayout,
    number: usize,
}

impl OrderedList {
    /// Creates a new ordered list starting at 1.
    pub fn new() -> OrderedList {
        OrderedList::with_start(1)
    }

    /// Creates a new ordered list with the given start number.
    pub fn with_start(start: usize) -> OrderedList {
        OrderedList {
            layout: LinearLayout::vertical(),
            number: start,
        }
    }

    /// Adds an element to this list.
    pub fn push<E: Element + 'static>(&mut self, element: E) {
        let mut point = BulletPoint::new(element);
        point.set_bullet(format!("{}.", self.number));
        self.layout.push(point);
        self.number += 1;
    }

    /// Adds an element to this list and returns the list.
    pub fn element<E: Element + 'static>(mut self, element: E) -> Self {
        self.push(element);
        self
    }
}

impl Element for OrderedList {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        self.layout.render(context, area, style)
    }
}

impl Default for OrderedList {
    fn default() -> OrderedList {
        OrderedList::new()
    }
}

/// A bullet point in a list.
///
/// This is a helper element for the [`OrderedList`][] and [`UnorderedList`][] types, but you can
/// also use it directly if you have special requirements.
///
/// # Example
///
/// ```
/// use genpdf::elements;
/// let layout = elements::LinearLayout::vertical()
///     .element(elements::BulletPoint::new(elements::Paragraph::new("first"))
///         .with_bullet("a)"))
///     .element(elements::BulletPoint::new(elements::Paragraph::new("second"))
///         .with_bullet("b)"));
/// ```
///
/// [`OrderedList`]: struct.OrderedList.html
/// [`UnorderedList`]: struct.UnorderedList.html
pub struct BulletPoint<E: Element> {
    element: E,
    indent: Mm,
    bullet_space: Mm,
    bullet: String,
    bullet_rendered: bool,
}

impl<E: Element> BulletPoint<E> {
    /// Creates a new bullet point with the given element.
    pub fn new(element: E) -> BulletPoint<E> {
        BulletPoint {
            element,
            indent: Mm::from(10),
            bullet_space: Mm::from(2),
            bullet: String::from("–"),
            bullet_rendered: false,
        }
    }

    /// Sets the bullet point symbol for this bullet point.
    pub fn set_bullet(&mut self, bullet: impl Into<String>) {
        self.bullet = bullet.into();
    }

    /// Sets the bullet point symbol for this bullet point and returns the bullet point.
    pub fn with_bullet(mut self, bullet: impl Into<String>) -> Self {
        self.set_bullet(bullet);
        self
    }
}

impl<E: Element> Element for BulletPoint<E> {
    fn render(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut element_area = area.clone();
        element_area.add_offset(Position::new(self.indent, 0));
        let mut result = self.element.render(context, element_area, style)?;
        result.size.width += self.indent;
        if !self.bullet_rendered {
            let bullet_width = style.str_width(&context.font_cache, &self.bullet);
            area.print_str(
                &context.font_cache,
                Position::new(self.indent - bullet_width - self.bullet_space, 0),
                style,
                self.bullet.as_str(),
            )?;
            self.bullet_rendered = true;
        }
        Ok(result)
    }
}

/// A decorator for table cells.
///
/// Implementations of this trait can be used to style cells of a [`TableLayout`][].
///
/// [`TableLayout`]: struct.TableLayout.html
pub trait CellDecorator {
    /// Sets the size of the table.
    ///
    /// This function is called once before the first call to [`decorate_cell`][].
    ///
    /// [`decorate_cell`]: #tymethod.decorate_cell
    fn set_table_size(&mut self, num_columns: usize, num_rows: usize) {
        let _ = (num_columns, num_rows);
    }

    /// Styles the cell with the given indizes thas has been rendered within the given area.
    fn decorate_cell(
        &mut self,
        column: usize,
        row: usize,
        has_more: bool,
        area: render::Area<'_>,
        style: Style,
    );
}

/// A cell decorator that draws frames around table cells.
///
/// This decorator draws frames around the cells of a [`TableLayout`][].  You can configure whether
/// inner, outer and continuation borders are drawn.  A continuation border is a border between a
/// cell and the page margin that occurs if a cell has to be wrapped to a new page.
///
/// [`TableLayout`]: struct.TableLayout.html
#[derive(Clone, Debug, Default)]
pub struct FrameCellDecorator {
    inner: bool,
    outer: bool,
    cont: bool,
    num_columns: usize,
    num_rows: usize,
    last_row: Option<usize>,
}

impl FrameCellDecorator {
    /// Creates a new frame cell decorator with the given settings for inner, outer and
    /// continuation borders.
    pub fn new(inner: bool, outer: bool, cont: bool) -> FrameCellDecorator {
        FrameCellDecorator {
            inner,
            outer,
            cont,
            ..Default::default()
        }
    }

    fn print_left(&self, column: usize) -> bool {
        if column == 0 {
            self.outer
        } else {
            self.inner
        }
    }

    fn print_right(&self, column: usize) -> bool {
        if column + 1 == self.num_columns {
            self.outer
        } else {
            self.inner
        }
    }

    fn print_top(&self, row: usize) -> bool {
        if self.last_row.map(|last_row| row > last_row).unwrap_or(true) {
            if row == 0 {
                self.outer
            } else {
                self.inner
            }
        } else {
            self.cont
        }
    }

    fn print_bottom(&self, row: usize, has_more: bool) -> bool {
        if has_more {
            self.cont
        } else if row + 1 == self.num_rows {
            self.outer
        } else {
            self.inner
        }
    }
}

impl CellDecorator for FrameCellDecorator {
    fn set_table_size(&mut self, num_columns: usize, num_rows: usize) {
        self.num_columns = num_columns;
        self.num_rows = num_rows;
    }

    fn decorate_cell(
        &mut self,
        column: usize,
        row: usize,
        has_more: bool,
        area: render::Area<'_>,
        style: Style,
    ) {
        let size = area.size();

        if self.print_left(column) {
            area.draw_line(
                vec![Position::default(), Position::new(0, size.height)],
                style,
            );
        }

        if self.print_right(column) {
            area.draw_line(
                vec![
                    Position::new(size.width, 0),
                    Position::new(size.width, size.height),
                ],
                style,
            );
        }

        if self.print_top(row) {
            area.draw_line(
                vec![Position::default(), Position::new(size.width, 0)],
                style,
            );
        }

        if self.print_bottom(row, has_more) {
            area.draw_line(
                vec![
                    Position::new(0, size.height),
                    Position::new(size.width, size.height),
                ],
                style,
            );
        }

        if column + 1 == self.num_columns {
            self.last_row = Some(row);
        }
    }
}

/// A row of a table layout.
///
/// This is a helper struct for populating a [`TableLayout`][].  After you have added all elements
/// to the row using [`push_element`][] or [`element`][], you can append the row to the table
/// layout by calling [`push`][].
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut table = elements::TableLayout::new(vec![1, 1]);
/// let mut row = table.row();
/// row.push_element(elements::Paragraph::new("Cell 1"));
/// row.push_element(elements::Paragraph::new("Cell 2"));
/// row.push().expect("Invalid table row");
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let table = elements::TableLayout::new(vec![1, 1])
///     .row()
///     .element(elements::Paragraph::new("Cell 1"))
///     .element(elements::Paragraph::new("Cell 2"))
///     .push()
///     .expect("Invalid table row");
/// ```
///
/// [`TableLayout`]: struct.TableLayout.html
/// [`push`]: #method.push
/// [`push_element`]: #method.push_element
/// [`element`]: #method.element
pub struct TableLayoutRow<'a> {
    table_layout: &'a mut TableLayout,
    elements: Vec<Box<dyn Element>>,
}

impl<'a> TableLayoutRow<'a> {
    fn new(table_layout: &'a mut TableLayout) -> TableLayoutRow<'a> {
        TableLayoutRow {
            table_layout,
            elements: Vec::new(),
        }
    }

    /// Adds the given element to this row.
    pub fn push_element<E: Element + 'static>(&mut self, element: E) {
        self.elements.push(Box::new(element));
    }

    /// Adds the given element to this row and returns the row.
    #[must_use]
    pub fn element<E: Element + 'static>(mut self, element: E) -> Self {
        self.push_element(element);
        self
    }

    /// Tries to append this row to the table.
    ///
    /// This method fails if the number of elements in this row does not match the number of
    /// columns in the table.
    pub fn push(self) -> Result<(), Error> {
        self.table_layout.push_row(self.elements)
    }
}

/// Arranges elements in columns and rows.
///
/// This struct can be used to layout arbitrary elements in columns in rows, or to draw typical
/// tables.  You can customize the cell style by providing a [`CellDecorator`][] implementation.
/// If you want to print a typical table with borders around the cells, use the
/// [`FrameCellDecorator`][].
///
/// The column widths are determined by the weights that have been set in the constructor.  The
/// table always uses the full width of the provided area.
///
/// # Examples
///
/// With setters:
/// ```
/// use genpdf::elements;
/// let mut table = elements::TableLayout::new(vec![1, 1]);
/// table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, false));
/// let mut row = table.row();
/// row.push_element(elements::Paragraph::new("Cell 1"));
/// row.push_element(elements::Paragraph::new("Cell 2"));
/// row.push().expect("Invalid table row");
/// ```
///
/// Chained:
/// ```
/// use genpdf::elements;
/// let table = elements::TableLayout::new(vec![1, 1])
///     .row()
///     .element(elements::Paragraph::new("Cell 1"))
///     .element(elements::Paragraph::new("Cell 2"))
///     .push()
///     .expect("Invalid table row");
/// ```
///
/// [`CellDecorator`]: trait.CellDecorator.html
/// [`FrameCellDecorator`]: struct.FrameCellDecorator.html
pub struct TableLayout {
    column_weights: Vec<usize>,
    rows: Vec<Vec<Box<dyn Element>>>,
    render_idx: usize,
    cell_decorator: Option<Box<dyn CellDecorator>>,
}

impl TableLayout {
    /// Creates a new table layout with the given column weights.
    ///
    /// The column weights are used to determine the relative width of the columns.  The number of
    /// column weights determines the number of columns in the table.
    pub fn new(column_weights: Vec<usize>) -> TableLayout {
        TableLayout {
            column_weights,
            rows: Vec::new(),
            render_idx: 0,
            cell_decorator: None,
        }
    }

    /// Sets the cell decorator for this table.
    pub fn set_cell_decorator(&mut self, decorator: impl CellDecorator + 'static) {
        self.cell_decorator = Some(Box::from(decorator));
    }

    /// Adds a row to this table using the [`TableLayoutRow`][] helper struct.
    ///
    /// [`TableLayoutRow`]: struct.TableLayoutRow.html
    pub fn row(&mut self) -> TableLayoutRow<'_> {
        TableLayoutRow::new(self)
    }

    /// Adds a row to this table.
    ///
    /// The number of elements in the given vector must match the number of columns.  Otherwise, an
    /// error is returned.
    pub fn push_row(&mut self, row: Vec<Box<dyn Element>>) -> Result<(), Error> {
        if row.len() == self.column_weights.len() {
            self.rows.push(row);
            Ok(())
        } else {
            Err(Error::new(
                format!(
                    "Expected {} elements in table row, received {}",
                    self.column_weights.len(),
                    row.len()
                ),
                ErrorKind::InvalidData,
            ))
        }
    }

    fn render_row(
        &mut self,
        context: &Context,
        area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();

        let areas = area.split_horizontally(&self.column_weights);
        let mut row_height = Mm::from(0);
        for (area, element) in areas.iter().zip(self.rows[self.render_idx].iter_mut()) {
            let element_result = element.render(context, area.clone(), style)?;
            result.has_more |= element_result.has_more;
            row_height = row_height.max(element_result.size.height);
        }
        result.size.height = row_height;

        if let Some(decorator) = &mut self.cell_decorator {
            for (i, mut area) in areas.into_iter().enumerate() {
                area.set_height(row_height);
                decorator.decorate_cell(i, self.render_idx, result.has_more, area, style);
            }
        }

        Ok(result)
    }
}

impl Element for TableLayout {
    fn render(
        &mut self,
        context: &Context,
        mut area: render::Area<'_>,
        style: Style,
    ) -> Result<RenderResult, Error> {
        let mut result = RenderResult::default();
        if self.column_weights.is_empty() {
            return Ok(result);
        }
        if let Some(decorator) = &mut self.cell_decorator {
            decorator.set_table_size(self.column_weights.len(), self.rows.len());
        }
        result.size.width = area.size().width;
        while self.render_idx < self.rows.len() {
            let row_result = self.render_row(context, area.clone(), style)?;
            result.size.height += row_result.size.height;
            area.add_offset(Position::new(0, row_result.size.height));
            if row_result.has_more {
                break;
            }
            self.render_idx += 1;
        }
        result.has_more = self.render_idx < self.rows.len();
        Ok(result)
    }
}
