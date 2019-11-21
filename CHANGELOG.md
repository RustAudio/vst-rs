# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.0

### Changed

- **Major breaking change:** Restructured `Plugin` API to make it thread safe ([#65](https://github.com/RustAudio/vst-rs/pull/65))
- Fixed a number of unsoundness issues in the `Outputs` API ([#67](https://github.com/RustAudio/vst-rs/pull/67), [#108](https://github.com/RustAudio/vst-rs/pull/108))
- Set parameters to be automatable by default ([#99](https://github.com/RustAudio/vst-rs/pull/99))
- Moved repository to the [RustAudio](https://github.com/RustAudio) organization and renamed it to `vst-rs` ([#90](https://github.com/RustAudio/vst-rs/pull/90), [#94](https://github.com/RustAudio/vst-rs/pull/94))

### Fixed

- Fixed a use-after-move bug in the event iterator ([#93](https://github.com/RustAudio/vst-rs/pull/93), [#111](https://github.com/RustAudio/vst-rs/pull/111))

### Added

- Handle `Opcode::GetEffectName` to resolve name display issues on some hosts ([#89](https://github.com/RustAudio/vst-rs/pull/89))
- More examples ([#65](https://github.com/RustAudio/vst-rs/pull/65), [#92](https://github.com/RustAudio/vst-rs/pull/92))

## 0.1.0

### Added

- Added initial changelog
- Initial project files

### Removed

- The `#[derive(Copy, Clone)]` attribute from `Outputs`.

### Changed
- The signature of the `Outputs::split_at_mut` now takes an `self` parameter instead of `&mut self`.
So calling `split_at_mut` will now move instead of "borrow".
- Now `&mut Outputs` (instead of `Outputs`) implements the `IntoIterator` trait.
- The return type of the `AudioBuffer::zip()` method (but it still implements the Iterator trait).
