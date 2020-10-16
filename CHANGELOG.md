<!---
SPDX-FileCopyrightText: 2020 Robin Krahl <robin.krahl@ireas.org>
SPDX-License-Identifier: CC0-1.0
-->

# Unreleased

## Breaking Changes

- Improve the font handling:
  - Make `FontFamily` generic over the font data type.

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
