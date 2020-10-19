// SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: Apache-2.0 or MIT

//! Fonts, font families and a font cache.
//!
//! Before you can use a font in a PDF document, you have to load the [`FontData`][] for it, either
//! from a file ([`FontData::load`][]) or from bytes ([`FontData::new`][]).  See the [`rusttype`][]
//! crate for the supported data formats.  Use the [`from_files`][] function to load a font family
//! from a set of files following the default naming conventions.
//!
//! The [`FontCache`][] caches all loaded fonts.  A [`Font`][] is a reference to a cached font in
//! the [`FontCache`][].  A [`FontFamily`][] is a collection of a regular, a bold, an italic and a
//! bold italic font (raw data or cached).
//!
//! Add fonts to a document’s font cache by calling [`Document::add_font_family`][].  This method
//! returns a reference to the cached data that you then can use with the [`Style`][] struct to
//! change the font family of an element.
//!
//! There are two methods for using fonts in a PDF font:  You can either embed the font data into
//! the PDF file.  Or you can use one of the three built-in font families ([`Builtin`][]) that PDF
//! viewers are expected to support.  You can choose between the two methods when loading the font
//! ([`from_files`][], [`FontData::load`][], [`FontData::new`][]).
//!
//! If you choose a built-in font family, you still have to provide the font data so that `genpdf`
//! has access to its glyph metrics.  Note that it is sufficient to use a font that is metrically
//! identical to the built-in font.  For example, you can use the Liberation fonts instad of the
//! proprietary Helvetica, Times and Courier fonts.
//!
//! Built-in fonts can only be used with characters that are supported by the [Windows-1252][]
//! encoding.
//!
//! **Note:**  The [`Font`][] and [`FontFamily<Font>`][`FontFamily`] structs are only valid for the
//! [`FontCache`][] they have been created with.  If you dont use the low-level [`render`][] module
//! directly, only use the [`Document::add_font_family`][] method to add fonts!
//!
//! # Internals
//!
//! There are two types of font data: A [`FontData`][] instance stores information about the glyph
//! metrics that is used to calculate the text size.  It can be loaded at any time using the
//! [`FontData::load`][] and [`FontData::new`][] methods.  Once the PDF document is rendered, a
//! [`printpdf::IndirectFontRef`][] is used to draw text in the PDF document.  Before a font can be
//! used in a PDF document, it has to be embedded using the [`FontCache::load_pdf_fonts`][] method.
//!
//! If you use the high-level interface provided by [`Document`][] to generate a PDF document, these
//! steps are done automatically.  You only have to manually populate the font cache if you use the
//! low-level interface in the [`render`][] module.
//!
//! [`render`]: ../render/
//! [`Document`]: ../struct.Document.html
//! [`Document::add_font_family`]: ../struct.Document.html#method.add_font_family
//! [`Style`]: ../style/struct.Style.html
//! [`from_files`]: fn.from_files.html
//! [`Builtin`]: enum.Builtin.html
//! [`FontCache`]: struct.FontCache.html
//! [`FontCache::load_pdf_fonts`]: struct.FontCache.html#method.load_pdf_fonts
//! [`FontData`]: struct.FontData.html
//! [`FontData::new`]: struct.FontData.html#method.new
//! [`FontData::load`]: struct.FontData.html#method.load
//! [`Font`]: struct.Font.html
//! [`FontFamily`]: struct.FontFamily.html
//! [`rusttype`]: https://docs.rs/rusttype
//! [`rusttype::Font`]: https://docs.rs/rusttype/0.8.3/rusttype/struct.Font.html
//! [`printpdf`]: https://docs.rs/printpdf
//! [`printpdf::IndirectFontRef`]: https://docs.rs/printpdf/0.3.2/printpdf/types/plugins/graphics/two_dimensional/font/struct.IndirectFontRef.html
//! [Windows-1252]: https://en.wikipedia.org/wiki/Windows-1252

use std::fmt;
use std::fs;
use std::path;

use crate::error::{Context as _, Error, ErrorKind};
use crate::render;
use crate::style::Style;
use crate::Mm;

/// Stores font data that can be referenced by a [`Font`][] or [`FontFamily`][].
///
/// If you use the high-level interface provided by [`Document`][], you don’t have to access this
/// type.  See the [module documentation](index.html) for details on the internals.
///
/// [`Document`]: ../struct.Document.html
/// [`Font`]: struct.Font.html
/// [`FontFamily`]: struct.FontFamily.html
#[derive(Debug)]
pub struct FontCache {
    fonts: Vec<FontData>,
    pdf_fonts: Vec<printpdf::IndirectFontRef>,
    // We have to use an option because we first have to construct the FontCache before we can load
    // a font, but the default font is always loaded in new, so this options is always some
    // (outside of new).
    default_font_family: Option<FontFamily<Font>>,
}

impl FontCache {
    /// Creates a new font cache with the given default font family.
    pub fn new(default_font_family: FontFamily<FontData>) -> FontCache {
        let mut font_cache = FontCache {
            fonts: Vec::new(),
            pdf_fonts: Vec::new(),
            default_font_family: None,
        };
        font_cache.default_font_family = Some(font_cache.add_font_family(default_font_family));
        font_cache
    }

    /// Adds the given font to the cache and returns a reference to it.
    pub fn add_font(&mut self, font_data: FontData) -> Font {
        let is_builtin = match &font_data.raw_data {
            RawFontData::Builtin(_) => true,
            RawFontData::Embedded(_) => false,
        };
        let font = Font::new(self.fonts.len(), is_builtin, &font_data.rt_font);
        self.fonts.push(font_data);
        font
    }

    /// Adds the given font family to the cache and returns a reference to it.
    pub fn add_font_family(&mut self, family: FontFamily<FontData>) -> FontFamily<Font> {
        FontFamily {
            regular: self.add_font(family.regular),
            bold: self.add_font(family.bold),
            italic: self.add_font(family.italic),
            bold_italic: self.add_font(family.bold_italic),
        }
    }

    /// Embeds all loaded fonts into the document generated by the given renderer and caches a
    /// reference to them.
    pub fn load_pdf_fonts(&mut self, renderer: &render::Renderer) -> Result<(), Error> {
        self.pdf_fonts.clear();
        for font in &self.fonts {
            let pdf_font = match &font.raw_data {
                RawFontData::Builtin(builtin) => renderer.add_builtin_font(*builtin)?,
                RawFontData::Embedded(data) => renderer.add_embedded_font(&data)?,
            };
            self.pdf_fonts.push(pdf_font);
        }
        Ok(())
    }

    /// Returns the default font family for this font cache.
    pub fn default_font_family(&self) -> FontFamily<Font> {
        self.default_font_family
            .expect("Invariant violated: no default font family for FontCache")
    }

    /// Returns a reference to the emebdded PDF font for the given font, if available.
    ///
    /// This method may only be called with [`Font`][] instances that have been created by this
    /// font cache.  PDF fonts are only avaiable if [`load_pdf_fonts`][] has been called.
    ///
    /// [`Font`]: struct.Font.html
    /// [`load_pdf_fonts`]: #method.load_pdf_fonts
    pub fn get_pdf_font(&self, font: Font) -> Option<&printpdf::IndirectFontRef> {
        self.pdf_fonts.get(font.idx)
    }

    /// Returns a reference to the Rusttype font for the given font, if available.
    ///
    /// This method may only be called with [`Font`][] instances that have been created by this
    /// font cache.
    ///
    /// [`Font`]: struct.Font.html
    pub fn get_rt_font(&self, font: Font) -> &rusttype::Font<'static> {
        &self.fonts[font.idx].rt_font
    }
}

/// The data for a font that is cached by a [`FontCache`][].
///
/// [`FontCache`]: struct.FontCache.html
#[derive(Clone, Debug)]
pub struct FontData {
    rt_font: rusttype::Font<'static>,
    raw_data: RawFontData,
}

impl FontData {
    /// Loads a font from the given data.
    ///
    /// The provided data must by readable by [`rusttype`][].  If `builtin` is set, a built-in PDF
    /// font is used instead of embedding the font in the PDF file (see the [module
    /// documentation](index.html) for more information).  In this case, the given font must be
    /// metrically identical to the built-in font.
    ///
    /// [`rusttype`]: https://docs.rs/rusttype
    pub fn new(data: Vec<u8>, builtin: Option<printpdf::BuiltinFont>) -> Result<FontData, Error> {
        let raw_data = if let Some(builtin) = builtin {
            RawFontData::Builtin(builtin)
        } else {
            RawFontData::Embedded(data.clone())
        };
        let rt_font = rusttype::Font::from_bytes(data).context("Failed to read rusttype font")?;
        if rt_font.units_per_em() == 0 {
            Err(Error::new(
                "The font is not scalable",
                ErrorKind::InvalidFont,
            ))
        } else {
            Ok(FontData { rt_font, raw_data })
        }
    }

    /// Loads the font at the given path.
    ///
    /// The path must point to a file that can be read by [`rusttype`][].  If `builtin` is set, a
    /// built-in PDF font is used instead of embedding the font in the PDF file (see the [module
    /// documentation](index.html) for more information).  In this case, the given font must be
    /// metrically identical to the built-in font.
    ///
    /// [`rusttype`]: https://docs.rs/rusttype
    pub fn load(
        path: impl AsRef<path::Path>,
        builtin: Option<printpdf::BuiltinFont>,
    ) -> Result<FontData, Error> {
        let data = fs::read(path.as_ref())
            .with_context(|| format!("Failed to open font file {}", path.as_ref().display()))?;
        FontData::new(data, builtin)
    }
}

#[derive(Clone, Debug)]
enum RawFontData {
    Builtin(printpdf::BuiltinFont),
    Embedded(Vec<u8>),
}

#[derive(Clone, Copy, Debug)]
enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl FontStyle {
    fn name(&self) -> &'static str {
        match self {
            FontStyle::Regular => "Regular",
            FontStyle::Bold => "Bold",
            FontStyle::Italic => "Italic",
            FontStyle::BoldItalic => "BoldItalic",
        }
    }
}

impl fmt::Display for FontStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A built-in font family.
///
/// A PDF viewer typically supports three font families that don’t have to be embedded into the PDF
/// file:  Times, Helvetica and Courier.
///
/// See the [module documentation](index.html) for more information.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Builtin {
    /// The Times font family.
    Times,
    /// The Helvetica font family.
    Helvetica,
    /// The Courier font family.
    Courier,
}

impl Builtin {
    fn style(&self, style: FontStyle) -> printpdf::BuiltinFont {
        match self {
            Builtin::Times => match style {
                FontStyle::Regular => printpdf::BuiltinFont::TimesRoman,
                FontStyle::Bold => printpdf::BuiltinFont::TimesBold,
                FontStyle::Italic => printpdf::BuiltinFont::TimesItalic,
                FontStyle::BoldItalic => printpdf::BuiltinFont::TimesBoldItalic,
            },
            Builtin::Helvetica => match style {
                FontStyle::Regular => printpdf::BuiltinFont::Helvetica,
                FontStyle::Bold => printpdf::BuiltinFont::HelveticaBold,
                FontStyle::Italic => printpdf::BuiltinFont::HelveticaOblique,
                FontStyle::BoldItalic => printpdf::BuiltinFont::HelveticaBoldOblique,
            },
            Builtin::Courier => match style {
                FontStyle::Regular => printpdf::BuiltinFont::Courier,
                FontStyle::Bold => printpdf::BuiltinFont::CourierBold,
                FontStyle::Italic => printpdf::BuiltinFont::CourierOblique,
                FontStyle::BoldItalic => printpdf::BuiltinFont::CourierBoldOblique,
            },
        }
    }
}

/// A collection of fonts with different styles.
///
/// See the [module documentation](index.html) for details on the internals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontFamily<T: Clone + fmt::Debug> {
    /// The regular variant of this font family.
    pub regular: T,
    /// The bold variant of this font family.
    pub bold: T,
    /// The italic variant of this font family.
    pub italic: T,
    /// The bold italic variant of this font family.
    pub bold_italic: T,
}

impl<T: Clone + Copy + fmt::Debug + PartialEq> FontFamily<T> {
    /// Returns the font for the given style.
    pub fn get(&self, style: Style) -> T {
        if style.is_bold() && style.is_italic() {
            self.bold_italic
        } else if style.is_bold() {
            self.bold
        } else if style.is_italic() {
            self.italic
        } else {
            self.regular
        }
    }
}

/// A reference to a font cached by a [`FontCache`][].
///
/// See the [module documentation](index.html) for details on the internals.
///
/// [`FontCache`]: struct.FontCache.html
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Font {
    idx: usize,
    is_builtin: bool,
    scale: rusttype::Scale,
    line_height: Mm,
    glyph_height: Mm,
}

impl Font {
    fn new(idx: usize, is_builtin: bool, rt_font: &rusttype::Font<'static>) -> Font {
        let units_per_em = rt_font.units_per_em();
        assert!(units_per_em != 0);

        let units_per_em = f32::from(units_per_em);
        let v_metrics = rt_font.v_metrics_unscaled();
        let glyph_height = (v_metrics.ascent - v_metrics.descent) / units_per_em;
        let scale = rusttype::Scale::uniform(glyph_height);

        let line_height = glyph_height + v_metrics.line_gap / units_per_em;

        Font {
            idx,
            is_builtin,
            scale,
            line_height: printpdf::Pt(f64::from(line_height)).into(),
            glyph_height: printpdf::Pt(f64::from(glyph_height)).into(),
        }
    }

    /// Returns whether this font is a built-in PDF font.
    pub fn is_builtin(&self) -> bool {
        self.is_builtin
    }

    /// Returns the line height for text with this font and the given font size.
    pub fn get_line_height(&self, font_size: u8) -> Mm {
        self.line_height * f64::from(font_size)
    }

    /// Returns the glyph height for text with this font and the given font size.
    pub fn glyph_height(&self, font_size: u8) -> Mm {
        self.glyph_height * f64::from(font_size)
    }

    /// Returns the width of a character with this font and the given font size.
    ///
    /// The given [`FontCache`][] must be the font cache that loaded this font.
    ///
    /// [`FontCache`]: struct.FontCache.html
    pub fn char_width(&self, font_cache: &FontCache, c: char, font_size: u8) -> Mm {
        let advance_width = font_cache
            .get_rt_font(*self)
            .glyph(c)
            .scaled(self.scale)
            .h_metrics()
            .advance_width;
        Mm::from(printpdf::Pt(f64::from(
            advance_width * f32::from(font_size),
        )))
    }

    /// Returns the width of a string with this font and the given font size.
    ///
    /// The given [`FontCache`][] must be the font cache that loaded this font.
    ///
    /// [`FontCache`]: struct.FontCache.html
    pub fn str_width(&self, font_cache: &FontCache, s: &str, font_size: u8) -> Mm {
        let str_width: Mm = font_cache
            .get_rt_font(*self)
            .glyphs_for(s.chars())
            .map(|g| g.scaled(self.scale).h_metrics().advance_width)
            .map(|w| Mm::from(printpdf::Pt(f64::from(w * f32::from(font_size)))))
            .sum();
        let kerning_width: Mm = self
            .kerning(font_cache, s.chars())
            .into_iter()
            .map(|val| val * f32::from(font_size))
            .map(|val| Mm::from(printpdf::Pt(f64::from(val))))
            .sum();
        str_width + kerning_width
    }

    /// Returns the kerning data for the given sequence of characters.
    ///
    /// The *i*-th value of the returned data is the amount of kerning to insert before the *i*-th
    /// character of the sequence.
    ///
    /// The given [`FontCache`][] must be the font cache that loaded this font.
    ///
    /// [`FontCache`]: struct.FontCache.html
    pub fn kerning<I>(&self, font_cache: &FontCache, iter: I) -> Vec<f32>
    where
        I: IntoIterator<Item = char>,
    {
        let font = font_cache.get_rt_font(*self);
        font.glyphs_for(iter.into_iter())
            .scan(None, |last, g| {
                let pos = if let Some(last) = last {
                    Some(font.pair_kerning(self.scale, *last, g.id()))
                } else {
                    Some(0.0)
                };
                *last = Some(g.id());
                pos
            })
            .collect()
    }

    /// Returns the glyphs IDs for the given sequence of characters.
    ///
    /// The given [`FontCache`][] must be the font cache that loaded this font.
    ///
    /// [`FontCache`]: struct.FontCache.html
    pub fn glyph_ids<I>(&self, font_cache: &FontCache, iter: I) -> Vec<u16>
    where
        I: IntoIterator<Item = char>,
    {
        let font = font_cache.get_rt_font(*self);
        font.glyphs_for(iter.into_iter())
            .map(|g| g.id().0 as u16)
            .collect()
    }
}

fn from_file(
    dir: impl AsRef<path::Path>,
    name: &str,
    style: FontStyle,
    builtin: Option<Builtin>,
) -> Result<FontData, Error> {
    let builtin = builtin.map(|b| b.style(style));
    FontData::load(
        &dir.as_ref().join(format!("{}-{}.ttf", name, style)),
        builtin,
    )
}

/// Loads the font family at the given path with the given name.
///
/// This method assumes that at the given path, these files exist and are valid font files:
/// - `{name}-Regular.ttf`
/// - `{name}-Bold.ttf`
/// - `{name}-Italic.ttf`
/// - `{name}-BoldItalic.ttf`
///
/// If `builtin` is set, built-in PDF fonts are used instead of embedding the fonts in the PDF file
/// (see the [module documentation](index.html) for more information).  In this case, the given
/// fonts must be metrically identical to the built-in fonts.
pub fn from_files(
    dir: impl AsRef<path::Path>,
    name: &str,
    builtin: Option<Builtin>,
) -> Result<FontFamily<FontData>, Error> {
    let dir = dir.as_ref();
    Ok(FontFamily {
        regular: from_file(dir, name, FontStyle::Regular, builtin)?,
        bold: from_file(dir, name, FontStyle::Bold, builtin)?,
        italic: from_file(dir, name, FontStyle::Italic, builtin)?,
        bold_italic: from_file(dir, name, FontStyle::BoldItalic, builtin)?,
    })
}
