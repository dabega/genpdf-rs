// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Low-level PDF rendering utilities.
//!
//! This module provides low-level abstractions over [`printpdf`][]:  A [`Renderer`][] creates a
//! document with one or more pages with different sizes.  A [`Page`][] has one or more layers, all
//! of the same size.  A [`Layer`][] can be used to access its [`Area`][].
//!
//! An [`Area`][] is a view on a full layer or on a part of a layer.  It can be used to print
//! lines and text.  For more advanced text formatting, you can create a [`TextSection`][] from an
//! [`Area`][].
//!
//! [`printpdf`]: https://docs.rs/printpdf/latest/printpdf
//! [`Renderer`]: struct.Renderer.html
//! [`Page`]: struct.Page.html
//! [`Layer`]: struct.Layer.html
//! [`Area`]: struct.Area.html
//! [`TextSection`]: struct.TextSection.html

use std::io;

use crate::error::{Context as _, Error, ErrorKind};
use crate::fonts;
use crate::style::{Color, Style};
use crate::{Margins, Mm, Position, Size};

/// Renders a PDF document with one or more pages.
///
/// This is a wrapper around a [`printpdf::PdfDocumentReference`][].
///
/// [`printpdf::PdfDocumentReference`]: https://docs.rs/printpdf/0.3.2/printpdf/types/pdf_document/struct.PdfDocumentReference.html
pub struct Renderer {
    doc: printpdf::PdfDocumentReference,
    // invariant: pages.len() >= 1
    pages: Vec<Page>,
}

impl Renderer {
    /// Creates a new PDF document renderer with one page of the given size and the given title.
    pub fn new(size: impl Into<Size>, title: impl AsRef<str>) -> Result<Renderer, Error> {
        let size = size.into();
        let (doc, page_idx, layer_idx) = printpdf::PdfDocument::new(
            title.as_ref(),
            size.width.into(),
            size.height.into(),
            "Layer 1",
        );
        let page_ref = doc.get_page(page_idx);
        let layer_ref = page_ref.get_layer(layer_idx);
        let page = Page::new(page_ref, layer_ref, size);

        Ok(Renderer {
            doc,
            pages: vec![page],
        })
    }

    /// Sets the PDF conformance for the generated PDF document.
    pub fn with_conformance(mut self, conformance: printpdf::PdfConformance) -> Self {
        self.doc = self.doc.with_conformance(conformance);
        self
    }

    /// Adds a new page with the given size to the document.
    pub fn add_page(&mut self, size: impl Into<Size>) {
        let size = size.into();
        let (page_idx, layer_idx) =
            self.doc
                .add_page(size.width.into(), size.height.into(), "Layer 1");
        let page_ref = self.doc.get_page(page_idx);
        let layer_ref = page_ref.get_layer(layer_idx);
        self.pages.push(Page::new(page_ref, layer_ref, size))
    }

    /// Returns the number of pages in this document.
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Returns a page of this document.
    pub fn get_page(&self, idx: usize) -> Option<&Page> {
        self.pages.get(idx)
    }

    /// Returns a mutable reference to a page of this document.
    pub fn get_page_mut(&mut self, idx: usize) -> Option<&mut Page> {
        self.pages.get_mut(idx)
    }

    /// Returns a mutable reference to the first page of this document.
    pub fn first_page(&self) -> &Page {
        &self.pages[0]
    }

    /// Returns the first page of this document.
    pub fn first_page_mut(&mut self) -> &mut Page {
        &mut self.pages[0]
    }

    /// Returns the last page of this document.
    pub fn last_page(&self) -> &Page {
        &self.pages[self.pages.len() - 1]
    }

    /// Returns a mutable reference to the last page of this document.
    pub fn last_page_mut(&mut self) -> &mut Page {
        let idx = self.pages.len() - 1;
        &mut self.pages[idx]
    }

    /// Loads the font from the given data, adds it to the generated document and returns a
    /// reference to it.
    pub fn add_builtin_font(
        &self,
        builtin: printpdf::BuiltinFont,
    ) -> Result<printpdf::IndirectFontRef, Error> {
        self.doc
            .add_builtin_font(builtin)
            .context("Failed to load PDF font")
    }

    /// Loads the font from the given data, adds it to the generated document and returns a
    /// reference to it.
    pub fn add_embedded_font(&self, data: &[u8]) -> Result<printpdf::IndirectFontRef, Error> {
        self.doc
            .add_external_font(data)
            .context("Failed to load PDF font")
    }

    /// Writes this PDF document to a writer.
    pub fn write(self, w: impl io::Write) -> Result<(), Error> {
        self.doc
            .save(&mut io::BufWriter::new(w))
            .context("Failed to save document")
    }
}

/// A page of a PDF document.
///
/// This is a wrapper around a [`printpdf::PdfPageReference`][].
///
/// [`printpdf::PdfPageReference`]: https://docs.rs/printpdf/0.3.2/printpdf/types/pdf_page/struct.PdfPageReference.html
pub struct Page {
    page: printpdf::PdfPageReference,
    size: Size,
    // invariant: layers.len() >= 1
    layers: Vec<Layer>,
}

impl Page {
    fn new(
        page: printpdf::PdfPageReference,
        layer: printpdf::PdfLayerReference,
        size: Size,
    ) -> Page {
        Page {
            page,
            size,
            layers: vec![Layer::new(layer, size)],
        }
    }

    /// Adds a new layer with the given name to the page.
    pub fn add_layer(&mut self, name: impl Into<String>) {
        let layer = self.page.add_layer(name);
        self.layers.push(Layer::new(layer, self.size));
    }

    /// Returns the number of layers on this page.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Returns a layer of this page.
    pub fn get_layer(&self, idx: usize) -> Option<&Layer> {
        self.layers.get(idx)
    }

    /// Returns the first layer of this page.
    pub fn first_layer(&self) -> &Layer {
        &self.layers[0]
    }

    /// Returns the last layer of this page.
    pub fn last_layer(&self) -> &Layer {
        &self.layers[self.layers.len() - 1]
    }
}

/// A layer of a page of a PDF document.
///
/// This is a wrapper around a [`printpdf::PdfLayerReference`][].
///
/// [`printpdf::PdfLayerReference`]: https://docs.rs/printpdf/0.3.2/printpdf/types/pdf_layer/struct.PdfLayerReference.html
pub struct Layer {
    layer: printpdf::PdfLayerReference,
    size: Size,
}

impl Layer {
    fn new(layer: printpdf::PdfLayerReference, size: Size) -> Layer {
        Layer { layer, size }
    }

    /// Returns a drawable area for this layer.
    pub fn area(&self) -> Area<'_> {
        Area::new(self, Position::default(), self.size)
    }

    /// Transforms the given position that is relative to the upper left corner of the layer to a
    /// position that is relative to the lower left corner of the layer (as used by `printpdf`).
    fn transform_position(&self, mut position: Position) -> Position {
        position.y = self.size.height - position.y;
        position
    }
}

/// A view on an area of a PDF layer that can be drawn on.
///
/// This struct provides access to the drawing methods of a [`printpdf::PdfLayerReference`][].  It
/// is defined by the layer that is drawn on and the origin and the size of the area.
///
/// [`printpdf::PdfLayerReference`]: https://docs.rs/printpdf/0.3.2/printpdf/types/pdf_layer/struct.PdfLayerReference.html
#[derive(Clone)]
pub struct Area<'a> {
    layer: &'a Layer,
    origin: Position,
    size: Size,
}

impl<'a> Area<'a> {
    fn new(layer: &'a Layer, origin: Position, size: Size) -> Area<'a> {
        Area {
            layer,
            origin,
            size,
        }
    }

    /// Reduces the size of the drawable area by the given margins.
    pub fn add_margins(&mut self, margins: impl Into<Margins>) {
        let margins = margins.into();
        self.origin.x += margins.left;
        self.origin.y += margins.top;
        self.size.width -= margins.left + margins.right;
        self.size.height -= margins.top + margins.bottom;
    }

    /// Returns the size of this area.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Adds the given offset to the area, reducing the drawable area.
    pub fn add_offset(&mut self, offset: impl Into<Position>) {
        let offset = offset.into();
        self.origin.x += offset.x;
        self.origin.y += offset.y;
        self.size.width -= offset.x;
        self.size.height -= offset.y;
    }

    /// Sets the size of this area.
    pub fn set_size(&mut self, size: impl Into<Size>) {
        self.size = size.into();
    }

    /// Sets the width of this area.
    pub fn set_width(&mut self, width: Mm) {
        self.size.width = width;
    }

    /// Sets the height of this area.
    pub fn set_height(&mut self, height: Mm) {
        self.size.height = height;
    }

    /// Splits this area horizontally using the given weights.
    ///
    /// The returned vector has the same number of elements as the provided slice.  The width of
    /// the *i*-th area is *width \* weights[i] / total_weight*, where *width* is the width of this
    /// area, and *total_weight* is the sum of all given weights.
    pub fn split_horizontally(&self, weights: &[usize]) -> Vec<Area<'a>> {
        let total_weight: usize = weights.iter().sum();
        let factor = self.size.width / total_weight as f64;
        let widths = weights.iter().map(|weight| factor * *weight as f64);
        let mut offset = Mm(0.0);
        let mut areas = Vec::new();
        for width in widths {
            let mut area = self.clone();
            area.origin.x += offset;
            area.size.width = width;
            areas.push(area);
            offset += width;
        }
        areas
    }

    /// Draws a line with the given points and the given style.
    ///
    /// Currently, this method only uses the color of the given style as the outline color (if set).
    /// The points are relative to the upper left corner of the area.
    pub fn draw_line(&self, points: Vec<Position>, style: Style) {
        let line_points: Vec<_> = points
            .into_iter()
            .map(|pos| (self.transform_position(pos).into(), false))
            .collect();
        let line = printpdf::Line {
            points: line_points,
            is_closed: false,
            has_fill: false,
            has_stroke: true,
            is_clipping_path: false,
        };
        if let Some(color) = style.color() {
            self.layer().set_outline_color(color.into());
        }
        self.layer().add_shape(line);
        if style.color().is_some() {
            self.layer().set_outline_color(Color::Rgb(0, 0, 0).into());
        }
    }

    /// Tries to draw the given string at the given position and returns `true` if the area was
    /// large enough to draw the string.
    ///
    /// The font cache must contain the PDF font for the font set in the style.  The position is
    /// relative to the upper left corner of the area.
    pub fn print_str<S: AsRef<str>>(
        &self,
        font_cache: &fonts::FontCache,
        position: Position,
        style: Style,
        s: S,
    ) -> Result<bool, Error> {
        if let Ok(mut section) = self.text_section(font_cache, position, style) {
            section.print_str(s, style)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Creates a new text section at the given position or returns an error if the text section
    /// does not fit in this area.
    ///
    /// The given style is only used to calculate the line height of the section.  The position is
    /// relative to the upper left corner of the area.  The font cache must contain the PDF font
    /// for all fonts printed with the text section.
    pub fn text_section<'f>(
        &self,
        font_cache: &'f fonts::FontCache,
        position: Position,
        style: Style,
    ) -> Result<TextSection<'_, 'f, 'a>, ()> {
        TextSection::new(font_cache, self, position, style)
    }

    /// Transforms the given position that is relative to the upper left corner of the area to a
    /// position that is relative to the lower left corner of its layer (as used by `printpdf`).
    fn transform_position(&self, mut position: Position) -> Position {
        position += self.origin;
        self.layer.transform_position(position)
    }

    fn layer(&self) -> &printpdf::PdfLayerReference {
        &self.layer.layer
    }
}

/// A text section that is drawn on an area of a PDF layer.
pub struct TextSection<'a, 'f, 'l> {
    font_cache: &'f fonts::FontCache,
    area: &'a Area<'l>,
    line_height: Mm,
    cursor: Position,
    fill_color: Option<Color>,
}

impl<'a, 'f, 'l> TextSection<'a, 'f, 'l> {
    fn new(
        font_cache: &'f fonts::FontCache,
        area: &'a Area<'l>,
        position: Position,
        style: Style,
    ) -> Result<TextSection<'a, 'f, 'l>, ()> {
        let height = style.font(font_cache).glyph_height(style.font_size());

        if position.y + height > area.size.height {
            return Err(());
        }

        let line_height = style.line_height(font_cache);
        let section = TextSection {
            font_cache,
            area,
            line_height,
            cursor: position,
            fill_color: None,
        };
        section.layer().begin_text_section();
        section.layer().set_line_height(line_height.0);
        let cursor = area.transform_position(position);
        section
            .layer()
            .set_text_cursor(cursor.x.into(), (cursor.y - height).into());
        Ok(section)
    }

    /// Tries to add a new line and returns `true` if the area was large enough to fit the new
    /// line.
    #[must_use]
    pub fn add_newline(&mut self) -> bool {
        if self.cursor.y + self.line_height > self.area.size.height {
            false
        } else {
            self.layer().add_line_break();
            self.cursor.y += self.line_height;
            true
        }
    }

    /// Prints the given string with the given style.
    ///
    /// The font cache for this text section must contain the PDF font for the given style.
    pub fn print_str(&mut self, s: impl AsRef<str>, style: Style) -> Result<(), Error> {
        let font = style.font(self.font_cache);

        let positions = font
            .kerning(self.font_cache, s.as_ref().chars())
            .into_iter()
            // Kerning is measured in 1/1000 em
            .map(|pos| pos * -1000.0)
            .map(|pos| pos as i64);
        let codepoints = if font.is_builtin() {
            // Built-in fonts always use the Windows-1252 encoding
            encode_win1252(s.as_ref())?
        } else {
            font.glyph_ids(&self.font_cache, s.as_ref().chars())
        };

        let font = self
            .font_cache
            .get_pdf_font(font)
            .expect("Could not find PDF font in font cache");
        if let Some(color) = style.color() {
            self.layer().set_fill_color(color.into());
        } else if self.fill_color.is_some() {
            self.layer().set_fill_color(Color::Rgb(0, 0, 0).into());
        }
        self.fill_color = style.color();
        self.layer().set_font(font, style.font_size().into());

        self.layer()
            .write_positioned_codepoints(positions.zip(codepoints.iter().copied()));
        Ok(())
    }

    fn layer(&self) -> &printpdf::PdfLayerReference {
        self.area.layer()
    }
}

impl<'a, 'f, 'l> Drop for TextSection<'a, 'f, 'l> {
    fn drop(&mut self) {
        if self.fill_color.is_some() {
            self.layer().set_fill_color(Color::Rgb(0, 0, 0).into());
        }
        self.layer().end_text_section();
    }
}

/// Encodes the given string using the Windows-1252 encoding for use with built-in PDF fonts,
/// returning an error if it contains unsupported characters.
fn encode_win1252(s: &str) -> Result<Vec<u16>, Error> {
    let bytes: Vec<_> = lopdf::Document::encode_text(Some("WinAnsiEncoding"), s)
        .into_iter()
        .map(u16::from)
        .collect();

    // Windows-1252 is a single-byte encoding, so one byte is one character.
    if bytes.len() != s.chars().count() {
        Err(Error::new(
            format!(
                "Tried to print a string with characters that are not supported by the \
                Windows-1252 encoding with a built-in font: {}",
                s
            ),
            ErrorKind::UnsupportedEncoding,
        ))
    } else {
        Ok(bytes)
    }
}
