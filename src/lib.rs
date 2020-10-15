// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! User-friendly PDF generator written in pure Rust.
//!
//! `genpdf` is a high-level PDF generator built ontop of [`printpdf`][] and [`rusttype`][].  It
//! takes care of the page layout and text alignment and renders a document tree into a PDF
//! document.  All of its dependencies are written in Rust, so you don’t need any pre-installed
//! libraries or tools.
//!
//! # Quickstart
//!
//! To generate a PDF document, create an instance of the [`Document`][] struct and add
//! [`Element`][] implementations to it.  Then call the [`Document::render_to_file`][] method to
//! render the document and to write it to a file.
//!
//! ```no_run
//! // Create a document and set the default font family
//! let mut doc = genpdf::Document::new("./fonts", "Liberation")
//!     .expect("Failed to create PDF document");
//! // Change the default settings
//! doc.set_margins(10);
//! doc.set_title("Demo document");
//! // Add one or more elements
//! doc.push(genpdf::elements::Paragraph::new("This is a demo document."));
//! // Render the document and write it to a file
//! doc.render_to_file("output.pdf").expect("Failed to write PDF file");
//! ```
//!
//! For a complete example with all supported elements, see the [`examples/demo.rs`][] file that
//! generates [this PDF document][].
//!
//! # Overview
//!
//! A [`Document`][] consists of a [`LinearLayout`][] that renders the added elements, a
//! [`FontCache`][] instance that keeps track of the loaded fonts and a collection of default
//! values for the text style and the page layout.
//!
//! When creating a [`Document`][] instance, you always have to set the default font family that
//! will be used for the document.  You can load additional fonts with the
//! [`Document::load_font_family`][] method.
//!
//! The style of a shape or text can be set using the [`Style`][] struct.  The style is inherited
//! within the document tree.  You can set the style of an element by wrapping it in a
//! [`StyledElement`][] (see the [`Element::styled`][] method) or – for text elements – with a
//! [`StyledString`][].
//!
//! For an overview of the available elements, see the [`elements`][] module.  You can also
//! create custom elements by implementing the [`Element`][] trait.
//!
//! The actual PDF document is generated from the elements that have been added to the document
//! once you call the [`Document::render`][] or [`Document::render_to_file`][] methods.  For
//! details on the rendering process, see the next section.
//!
//! In `genpdf`, all lengths are measured in millimeters.  The only exceptions are font sizes that
//! are measured in points.  The [`Mm`][] newtype struct is used for all lengths, and the
//! [`Position`][] and [`Size`][] types are used to describe points and rectangles in the PDF
//! document.
//!
//! # Rendering Process
//!
//! The rendering process is started by calling the [`Document::render`][] or
//! [`Document::render_to_file`][] methods.  You can only render a document once.  Before the
//! rendering starts, the PDF document is created and all loaded fonts are embedded into the
//! document.
//!
//! The elements are then rendered by calling the [`Element::render`][] method of the root element,
//! a [`LinearLayout`][].  This element will then call the `render` methods of the elements stored
//! in the layout, and so on.
//!
//! The [`Element::render`][] method receives the following arguments:
//! - *font_cache* is the [`FontCache`][] instance that keeps track of the loaded fonts and can be
//!   used to map a [`Style`][] instance to font data.
//! - *area* is a view on the area of the current page that can be used by the element.
//! - *style* is the [`Style`][] instance for this element.  Is is a combination of the default
//!   style of the [`Document`][] and the style set by [`StyledElement`][] instances that are
//!   parents of the current element.
//!
//! The `render` method tries to render the entire element in the provided area.  The returned
//! [`RenderResult`][] stores the size of the area that has actually been used to render the
//! element.  If the element did not fit into the provided area, the `has_more` field of the
//! [`RenderResult`][] is set to `true`.  This causes the `Document` to add a new page to the PDF
//! document and then call the `render` method again with an area of the new page.  This is
//! repeated until all elements have been rendered completely, that means until all elements return
//! a [`RenderResult`][] with `has_more == false`.
//!
//! Elements may print to the provided area using the methods of the [`Area`][] struct, or by
//! calling the `render` method of other elements, or both.
//!
//! The render process is cancelled if an `Element` returns an error, or if no content has been
//! rendered to a newly created page.  This indicates that an element does not fit on a clear page
//! and can’t even be rendered partially, so the rendering process is cancelled.
//!
//! As the [`Element::render`][] method is called repeatedly until the complete element has been
//! rendered, the element has to keep track of the content that has already been rendered.  As
//! there is only one rendering process per document, elements may discard data that has been
//! rendered and that is no longer needed.
//!
//! # Low-Level Interface
//!
//! The [`render`][] module contains a low-level interface for creating PDF files.  It keeps track
//! of page sizes and layers and has utility methods for easier text and shape rendering.  But it
//! does not provide support for measuring the size of rendered text or for laying out elements.
//! If possible, you should always try to use `genpdf`’s high-level interface and implement the
//! [`Element`][] trait if you want to customize a document instead of using the low-level
//! interface directly.
//!
//! # Known Issues
//!
//! - Currently, `genpdf` adds all loaded fonts to the PDF document, even if they are not used.
//!   `printpdf` then adds all available glyphs for these fonts to the document, even if they are
//!   not used in the document.  This increases the file size by 100–200 KiB per font (500–1000 KiB
//!   per font family).  Until this is fixed, you can pass the generated file through `ps2pdf` to
//!   significantly reduce its size.
//!
//! [`printpdf`]: https://docs.rs/printpdf
//! [`rusttype`]: https://docs.rs/rusttype
//! [`render`]: ./render/
//! [`elements`]: ./elements/
//! [`Document`]: struct.Document.html
//! [`Document::render`]: struct.Document.html#method.render
//! [`Document::render_to_file`]: struct.Document.html#method.render_to_file
//! [`Document::load_font_family`]: struct.Document.html#method.load_font_family
//! [`Element`]: trait.Element.html
//! [`Element::render`]: trait.Element.html#tymethod.render
//! [`Element::styled`]: trait.Element.html#tymethod.styled
//! [`RenderResult`]: trait.RenderResult.html
//! [`LinearLayout`]: elements/struct.LinearLayout.html
//! [`StyledElement`]: elements/StyledElement.html
//! [`FontCache`]: fonts/struct.FontCache.html
//! [`Area`]: render/struct.Area.html
//! [`Mm`]: struct.Mm.html
//! [`Size`]: struct.Size.html
//! [`Position`]: struct.Position.html
//! [`Style`]: style/struct.Style.html
//! [`StyledString`]: style/struct.StyledString.html
//! [`examples/demo.rs`]: https://git.sr.ht/~ireas/genpdf-rs/tree/master/examples/demo.rs
//! [this PDF document]: https://git.sr.ht/~ireas/genpdf-rs/blob/master/examples/demo.pdf

#![warn(missing_docs, rust_2018_idioms)]

mod wrap;

pub mod elements;
pub mod error;
pub mod fonts;
pub mod render;
pub mod style;

use std::fs;
use std::io;
use std::path;
use std::str;

use derive_more::{
    Add, AddAssign, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign, Sum,
};

use crate::error::{Error, ErrorKind};

/// A length measured in millimeters.
///
/// `genpdf` always uses millimeters as its length unit, except for the font size that is measured
/// in points.
///
/// If you want to convert pixels or points into millimeters, you can use the [`printpdf::Pt`][]
/// and [`printpdf::Px`][] types.
///
/// [`printpdf::Pt`]: https://docs.rs/printpdf/0.3.2/printpdf/scale/struct.Pt.html
/// [`printpdf::Px`]: https://docs.rs/printpdf/0.3.2/printpdf/scale/struct.Px.html
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    PartialOrd,
    Add,
    AddAssign,
    Div,
    DivAssign,
    From,
    Into,
    Mul,
    MulAssign,
    Sub,
    SubAssign,
    Sum,
)]
pub struct Mm(f64);

impl Mm {
    /// Returns the maximum of this value and the given value.
    pub fn max(self, other: Mm) -> Mm {
        Mm(self.0.max(other.0))
    }
}

impl From<i8> for Mm {
    fn from(mm: i8) -> Mm {
        Mm(mm.into())
    }
}

impl From<i16> for Mm {
    fn from(mm: i16) -> Mm {
        Mm(mm.into())
    }
}

impl From<i32> for Mm {
    fn from(mm: i32) -> Mm {
        Mm(mm.into())
    }
}

impl From<u8> for Mm {
    fn from(mm: u8) -> Mm {
        Mm(mm.into())
    }
}

impl From<u16> for Mm {
    fn from(mm: u16) -> Mm {
        Mm(mm.into())
    }
}

impl From<u32> for Mm {
    fn from(mm: u32) -> Mm {
        Mm(mm.into())
    }
}

impl From<f32> for Mm {
    fn from(mm: f32) -> Mm {
        Mm(mm.into())
    }
}

impl From<printpdf::Mm> for Mm {
    fn from(mm: printpdf::Mm) -> Mm {
        Mm(mm.0)
    }
}

impl From<printpdf::Pt> for Mm {
    fn from(pt: printpdf::Pt) -> Mm {
        let mm: printpdf::Mm = pt.into();
        mm.into()
    }
}

impl From<Mm> for printpdf::Mm {
    fn from(mm: Mm) -> printpdf::Mm {
        printpdf::Mm(mm.0)
    }
}

impl From<Mm> for printpdf::Pt {
    fn from(mm: Mm) -> printpdf::Pt {
        printpdf::Mm(mm.0).into()
    }
}

/// A position on a PDF layer, measured in millimeters.
///
/// All positions used by `genpdf` are measured from the top left corner of the reference area.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Add, AddAssign, Sub, SubAssign)]
pub struct Position {
    /// The x coordinate of the position, measured from the left border of the reference area.
    pub x: Mm,
    /// The y coordinate of the position, measured from the top border of the reference area.
    pub y: Mm,
}

impl Position {
    /// Creates a new position from the given coordinates.
    pub fn new(x: impl Into<Mm>, y: impl Into<Mm>) -> Position {
        Position {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl<X: Into<Mm>, Y: Into<Mm>> From<(X, Y)> for Position {
    fn from(values: (X, Y)) -> Position {
        Position::new(values.0, values.1)
    }
}

/// A size of an area on a PDF layer, measured in millimeters.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Add, AddAssign, Sub, SubAssign)]
pub struct Size {
    /// The width of the area.
    pub width: Mm,
    /// The height of the area.
    pub height: Mm,
}

impl Size {
    /// Creates a new size from the given width and height.
    pub fn new(width: impl Into<Mm>, height: impl Into<Mm>) -> Size {
        Size {
            width: width.into(),
            height: height.into(),
        }
    }

    /// Stacks the given size vertically on this size and returns the result.
    ///
    /// This means that the width is set to the maximum of the widths and the height is set to the
    /// sum of the heights.
    #[must_use]
    pub fn stack_vertical(mut self, other: Size) -> Size {
        self.width = self.width.max(other.width);
        self.height += other.height;
        self
    }
}

impl<W: Into<Mm>, H: Into<Mm>> From<(W, H)> for Size {
    fn from(values: (W, H)) -> Size {
        Size::new(values.0, values.1)
    }
}

/// A paper size like A4, legal or letter.
///
/// This enum provides variants for typical paper sizes that can be converted into [`Size`][]
/// instances.
///
/// [`Size`]: struct.Size.html
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PaperSize {
    /// The A4 paper size (210x297mm).
    A4,
    /// The legal paper size (216x356mm).
    Legal,
    /// The letter paper size (216x279mm).
    Letter,
}

impl From<PaperSize> for Size {
    fn from(size: PaperSize) -> Size {
        match size {
            PaperSize::A4 => Size::new(210, 297),
            PaperSize::Legal => Size::new(216, 356),
            PaperSize::Letter => Size::new(216, 279),
        }
    }
}

/// The margins of an area, measured in millimeters.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Margins {
    /// The top margin of the area.
    top: Mm,
    /// The right margin of the area.
    right: Mm,
    /// The bottom margin of the area.
    bottom: Mm,
    /// The left margin of the area.
    left: Mm,
}

impl Margins {
    /// Creates a new `Margins` instance from the given top, right, bottom and left margins.
    pub fn trbl(
        top: impl Into<Mm>,
        right: impl Into<Mm>,
        bottom: impl Into<Mm>,
        left: impl Into<Mm>,
    ) -> Margins {
        Margins {
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
            left: left.into(),
        }
    }

    /// Creates a new `Margins` instance from the given vertical (top and bottom) and horizontal
    /// (left and right) margins.
    pub fn vh(vertical: impl Into<Mm>, horizontal: impl Into<Mm>) -> Margins {
        let (vertical, horizontal) = (vertical.into(), horizontal.into());
        Margins::trbl(vertical, horizontal, vertical, horizontal)
    }

    /// Creates a new `Margins` instance with all four margins set to the given value.
    pub fn all(all: impl Into<Mm>) -> Margins {
        let all = all.into();
        Margins::trbl(all, all, all, all)
    }
}

impl<T: Into<Mm>, R: Into<Mm>, B: Into<Mm>, L: Into<Mm>> From<(T, R, B, L)> for Margins {
    fn from(values: (T, R, B, L)) -> Margins {
        Margins::trbl(values.0, values.1, values.2, values.3)
    }
}

impl<V: Into<Mm>, H: Into<Mm>> From<(V, H)> for Margins {
    fn from(values: (V, H)) -> Margins {
        Margins::vh(values.0, values.1)
    }
}

impl<T: Into<Mm>> From<T> for Margins {
    fn from(value: T) -> Margins {
        Margins::all(value)
    }
}

/// A PDF document.
///
/// This struct is the entry point for the high-level `genpdf` API.  It stores a set of elements
/// and default style and layout settings.  Add elements to the document by calling the [`push`][]
/// method and then render them to a PDF file using the [`render`][] and [`render_to_file`][]
/// methods.
///
/// The root element of the document is a [`LinearLayout`][] that vertically arranges all elements.
/// For details on the rendering process, see the [Rendering Process section of the crate
/// documentation](index.html#rendering-process).
///
/// # Example
///
/// ```no_run
/// let mut doc = genpdf::Document::new("./fonts", "Liberation")
///     .expect("Failed to create document");
/// doc.push(genpdf::elements::Paragraph::new("Document content"));
/// doc.render_to_file("output.pdf").expect("Failed to render document");
/// ```
///
/// [`push`]: #method.push
/// [`render`]: #method.render
/// [`render_to_file`]: #method.render_to_file
/// [`LinearLayout`]: elements/struct.LinearLayout.html
pub struct Document {
    root: elements::LinearLayout,
    title: String,
    font_cache: fonts::FontCache,
    style: style::Style,
    paper_size: Size,
    margins: Option<Margins>,
    conformance: Option<printpdf::PdfConformance>,
}

impl Document {
    /// Creates a new document with the font family with the given name in the given directory as
    /// the default font.
    ///
    /// See the documentation of the [`load_font_family`][] method for details on the font loading.
    ///
    /// [`load_font_family`]: #method.load_font_family
    pub fn new(
        font_dir: impl AsRef<path::Path>,
        font_name: impl AsRef<str>,
    ) -> Result<Document, Error> {
        Ok(Document {
            root: elements::LinearLayout::vertical(),
            title: String::new(),
            font_cache: fonts::FontCache::new(font_dir, font_name.as_ref())?,
            style: style::Style::new(),
            paper_size: PaperSize::A4.into(),
            margins: None,
            conformance: None,
        })
    }

    /// Loads the font family with the given name from the given directory, adds them to the font
    /// cache and returns a reference to the loaded font family.
    ///
    /// The font is loaded from TTF files in the given directory.  It assumes that the following
    /// files exist and are valid font files:
    /// - `{dir}/{name}-Regular.ttf`
    /// - `{dir}/{name}-Bold.ttf`
    /// - `{dir}/{name}-Italic.ttf`
    /// - `{dir}/{name}-BoldItalic.ttf`
    ///
    /// Note that the returned font reference may only be used for this document.  It cannot be
    /// shared with other `Document` or [`FontCache`][] instances.
    ///
    /// [`FontCache`]: fonts/struct.FontCache.html
    pub fn load_font_family(
        &mut self,
        dir: impl AsRef<path::Path>,
        name: &str,
    ) -> Result<fonts::FontFamily, Error> {
        self.font_cache.load_font_family(dir, name)
    }

    /// Sets the title of the PDF document.
    ///
    /// If this method is not called, the PDF title will be empty.
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Sets the default font size in points for this document.
    ///
    /// If this method is not called, the default value of 12 points is used.
    pub fn set_font_size(&mut self, font_size: u8) {
        self.style.set_font_size(font_size);
    }

    /// Sets the default line spacing factor for this document.
    ///
    /// If this method is not called, the default value of 1 is used.
    pub fn set_line_spacing(&mut self, line_spacing: f64) {
        self.style.set_line_spacing(line_spacing);
    }

    /// Sets the paper size for all pages of this document.
    ///
    /// If this method is not called, the default size [`A4`][] is used.
    ///
    /// [`A4`]: enum.PaperSize.html#variant.A4
    pub fn set_paper_size(&mut self, paper_size: impl Into<Size>) {
        self.paper_size = paper_size.into();
    }

    /// Sets the margins for all pages of this document.
    ///
    /// If this method is not called, the full page is used.
    pub fn set_margins(&mut self, margins: impl Into<Margins>) {
        self.margins = Some(margins.into());
    }

    /// Sets the PDF conformance settings for this document.
    pub fn set_conformance(&mut self, conformance: printpdf::PdfConformance) {
        self.conformance = Some(conformance);
    }

    /// Sets the minimal PDF conformance settings for this document.
    ///
    /// If this method is called, the generation of ICC profiles and XMP metadata is deactivated,
    /// leading to a smaller file size.
    pub fn set_minimal_conformance(&mut self) {
        self.set_conformance(printpdf::PdfConformance::Custom(
            printpdf::CustomPdfConformance {
                requires_icc_profile: false,
                requires_xmp_metadata: false,
                ..Default::default()
            },
        ));
    }

    /// Adds the given element to the document.
    ///
    /// The given element is appended to the list of elements that is rendered by the root
    /// [`LinearLayout`][] once [`render`][] or [`render_to_file`][] is called.
    ///
    /// [`LinearLayout`]: elements/struct.LinearLayout.html
    /// [`render`]: #method.render
    /// [`render_to_file`]: #method.render_to_file
    pub fn push<E: Element + 'static>(&mut self, element: E) {
        self.root.push(element);
    }

    /// Renders this document into a PDF file and writes it to the given writer.
    ///
    /// The given writer is always wrapped in a buffered writer.  For details on the rendering
    /// process, see the [Rendering Process section of the crate
    /// documentation](index.html#rendering-process).
    pub fn render(mut self, w: impl io::Write) -> Result<(), Error> {
        let mut renderer = render::Renderer::new(self.paper_size, &self.title)?;
        if let Some(conformance) = self.conformance {
            renderer = renderer.with_conformance(conformance);
        }
        self.font_cache.load_pdf_fonts(&renderer)?;
        loop {
            let mut area = renderer.last_page().last_layer().area();
            if let Some(margins) = self.margins {
                area.add_margins(margins);
            }
            let result = self.root.render(&self.font_cache, area, self.style)?;
            if result.has_more {
                if result.size == Size::new(0, 0) {
                    return Err(Error::new(
                        "Could not fit an element on a new page",
                        ErrorKind::PageSizeExceeded,
                    ));
                }
                renderer.add_page(PaperSize::A4);
            } else {
                break;
            }
        }
        renderer.write(w)
    }

    /// Renders this document into a PDF file at the given path.
    ///
    /// If the given file does not exist, it is created.  If it exists, it is overwritten.
    ///
    /// For details on the rendering process, see the [Rendering Process section of the crate
    /// documentation](index.html#rendering-process).
    pub fn render_to_file(self, path: impl AsRef<path::Path>) -> Result<(), Error> {
        let path = path.as_ref();
        let file = fs::File::create(path)
            .map_err(|err| Error::new(format!("Could not create file {}", path.display()), err))?;
        self.render(file)
    }
}

/// The result of the rendering process.
///
/// This struct is returned by implementations of the [`Element::render`][] method.  It contains
/// the size of the area that has been written to (measured from the origin of the area that was
/// provided to the render method) and information about additional content that did not fit in the
/// provided area.
///
/// See the [Rendering Process section of the crate documentation](index.html#rendering-process)
/// for more information on the rendering process.
///
/// [`Element::render`]: trait.Element.html#tymethod.render
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct RenderResult {
    /// The size of the area that has been written to, starting from the origin of the provided
    /// area.
    pub size: Size,
    /// Indicates whether the element contains more content that did not fit in the provided area.
    pub has_more: bool,
}

/// An element of a PDF document.
///
/// This trait is implemented by all elements that can be added to a [`Document`][].  Implementors
/// have to define the [`render`][] method that writes the content of this element to the generated
/// PDF document.
///
/// See the [Rendering Process section of the crate documentation](index.html#rendering-process)
/// for more information on the rendering process.
///
/// [`Document`]: struct.Document.html
/// [`render`]: #tymethod.render
pub trait Element {
    /// Renders this element to the given area using the given style and font cache.
    ///
    /// For an overview over the rendering process, see the [Rendering Process section of the crate
    /// documentation](index.html#rendering-process).
    ///
    /// This method is called once for every element that has been added to a [`Document`][] once
    /// the [`render`][] or [`render_to_file`][] methods have been called.  If this method is
    /// called, it should print the element’s content to the given area.  If the content does not
    /// fit in the given area, it should set the `has_more` flag of the returned
    /// [`RenderResult`][].  It will then be called again with a new area on a new page until it
    /// returns a [`RenderResult`][] with `has_more == false`.  Regardless of whether the content
    /// fitted in the area or not, the `size` field of the [`RenderResult`][] must always be set to
    /// the size of the area that has been used, starting at the origin of the provided area.
    ///
    /// The following guarantuees are made by `genpdf`’s elements and must be followed by
    /// implementations of this trait:
    ///
    /// - There is only one rendering process per element instance.  This means that the first call
    ///   to this method is always the start of the rendering process, and subsequent calls are
    ///   always continuations of the same rendering process.  This means that the element does not
    ///   have to reset its state after it has processed all content, and it is allowed to drop
    ///   content that has already been rendered.
    /// - If a call to this method returns an `Err` value, it will not be called again.
    /// - After the first call, the method will only be called again if the `has_more` of the last
    ///   [`RenderResult`][] was set to true.
    /// - If none of the element’s content could be fitted in the provided area, the size of the
    ///   [`RenderResult`][] must be `(0, 0)`.  If the size is non-zero, this method must return a
    ///   [`RenderResult`] with `has_more == false` after a finite number of calls.
    ///
    /// [`Document`]: struct.Document.html
    /// [`render`]: struct.Document.html#method.render
    /// [`render_to_file`]: struct.Document.html#method.render_to_file
    /// [`RenderResult`]: struct.RenderResult.html
    fn render(
        &mut self,
        font_cache: &fonts::FontCache,
        area: render::Area<'_>,
        style: style::Style,
    ) -> Result<RenderResult, Error>;

    /// Draws a frame around this element.
    fn framed(self) -> elements::FramedElement<Self>
    where
        Self: Sized,
    {
        elements::FramedElement::new(self)
    }

    /// Adds a padding to this element.
    fn padded(self, padding: impl Into<Margins>) -> elements::PaddedElement<Self>
    where
        Self: Sized,
    {
        elements::PaddedElement::new(self, padding)
    }

    /// Sets the default style for this element and its children.
    fn styled(self, style: impl Into<style::Style>) -> elements::StyledElement<Self>
    where
        Self: Sized,
    {
        elements::StyledElement::new(self, style.into())
    }
}
