<!---
SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
SPDX-License-Identifier: CC0-1.0
-->

# Unreleased

## Breaking Changes

- Improve the font handling:
  - Make `FontFamily` generic over the font data type.
  - Make the fields of the `FontFamily` struct public.
  - Load the PDF font in `Renderer::load_font` from bytes instead of a path.
  - Separate font loading and font caching:
    - Replace the `load_font` and `load_font_family` methods of the `FontCache`
      struct with `add_font` and `add_font_family`, and the `load_font_family`
      method of `Document` with `add_font_family`.
    - Add the `FontData::load` method and the `fonts::load_from_files`
      function.
    - Change the arguments of the `FontCache::new` and `Decorator::new`
      methods.
  - Make the `Document::new`, `Document::add_font_family`, `FontCache::new`,
    `FontCache::add_font`, `FontCache::add_font_family` and `Font::new` methods
    infallible.
  - Add support for built-in fonts.
    - Added the `Error::UnsupportedEncoding` variant.
    - Change the return type of the `Area::print_str` and
      `TextSection::print_str` methods to return a `Result`.
- Move the `FontCache` instance used during the rendering process to the new
  `Context` struct.
- Remove the `Document::set_margins` method (use a `PageDecorator` instead).

## Non-Breaking Changes

- Add the `StyledCow<'s>` struct that contains a `Cow<'s, str>` with a `Style`
  annotation.
- Derive `Copy` for `StyledStr`.
- Add support for hyphenation (enabled by the `hyphenation` feature).
- Add the `PageBreak` element.
- Implement `From<Vec<StyledString>>` for `Paragraph`.
- Add the `PageDecorator` trait, the `SimplePageDecorator` implementation and
  the `Document::set_page_decorator` method to allow customization of all
  document pages.
- Add support for kerning and add the `Font::kerning` and `Font::glyph_ids`
  methods.

## Bug Fixes

- Always use the configured paper size when adding new pages to a `Document`.

# v0.1.1 (2020-10-16)

This patch release adds some trait implementations and utility functions and
improves the crate documentation.

Thanks to Matteo Bertini for contributions.

- Implement `From<&String>` for `StyledString`.
- Derive `Add`, `AddAssign`, `Sub` and `SubAssign` for `Position` and `Size`.
- Implement `From<Position>` for `printpdf::Point`.
- Add `split_horizontally` method to `Area`.
- Add `width` method to `StyledString` and `StyledStr`.
- Add `font_cache` method to `Document`.

# v0.1.0 (2020-10-15)

Initial release with support for formatted text, text wrapping and basic
shapes.
