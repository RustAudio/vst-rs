# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.0

### Changed

- Added deprecation notice.

## 0.3.0

### Fixed

- `SysExEvent` no longer contains invalid data on 64-bit systems ([#170](https://github.com/RustAudio/vst-rs/pull/171)]
- Function pointers in `AEffect` marked as `extern` ([#141](https://github.com/RustAudio/vst-rs/pull/141))
- Key character fixes ([#152](https://github.com/RustAudio/vst-rs/pull/152))
- Doc and deploy actions fixes ([9eb1bef](https://github.com/RustAudio/vst-rs/commit/9eb1bef1826db1581b4162081de05c1090935afb))
- Various doc fixes ([#177](https://github.com/RustAudio/vst-rs/pull/177))

### Added

- `begin_edit` and `end_edit` now in `Host` trait ([#151](https://github.com/RustAudio/vst-rs/pull/151))
- Added a `prelude` for commonly used items when constructing a `Plugin` ([#161](https://github.com/RustAudio/vst-rs/pull/161))
- Various useful implementations for `AtomicFloat` ([#150](https://github.com/RustAudio/vst-rs/pull/150))

### Changed

- **Major breaking change:** New `Plugin` `Send` requirement ([#140](https://github.com/RustAudio/vst-rs/pull/140))
- No longer require `Plugin` to implement `Default` ([#154](https://github.com/RustAudio/vst-rs/pull/154))
- `impl_clicke` replaced with `num_enum` ([#168](https://github.com/RustAudio/vst-rs/pull/168))
- Reworked `SendEventBuffer` to make it useable in `Plugin::process_events` ([#160](https://github.com/RustAudio/vst-rs/pull/160))
- Updated dependencies and removed development dependency on `time` ([#179](https://github.com/RustAudio/vst-rs/pull/179))

## 0.2.1

### Fixed

- Introduced zero-valued `EventType` variant to enable zero-initialization of `Event`, fixing a panic on Rust 1.48 and newer ([#138](https://github.com/RustAudio/vst-rs/pull/138))
- `EditorGetRect` opcode returns `1` on success, ensuring that the provided dimensions are applied by the host ([#115](https://github.com/RustAudio/vst-rs/pull/115))

### Added

- Added `update_display()` method to `Host`, telling the host to update its display (after a parameter change) via the `UpdateDisplay` opcode ([#126](https://github.com/RustAudio/vst-rs/pull/126))
- Allow plug-in to return a custom value in `can_do()` via the `Supported::Custom` enum variant ([#130](https://github.com/RustAudio/vst-rs/pull/130))
- Added `PartialEq` and `Eq` for `Supported` ([#135](https://github.com/RustAudio/vst-rs/pull/135))
- Implemented `get_editor()` and `Editor` interface for `PluginInstance` to enable editor support on the host side ([#136](https://github.com/RustAudio/vst-rs/pull/136))
- Default value (`0.0`) for `AtomicFloat` ([#139](https://github.com/RustAudio/vst-rs/pull/139))

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
