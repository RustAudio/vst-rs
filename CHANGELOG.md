# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added initial changelog

### Removed

- The `#[derive(Copy, Clone)]` attribute from `Outputs`.

### Changed
- The signature of the `Outputs::split_at_mut` now takes an `self` parameter instead of `&mut self`.
So calling `split_at_mut` will now move instead of "borrow".
- Now `&mut Outputs` (instead of `Outputs`) implements the `IntoIterator` trait.
- The return type of the `AudioBuffer::zip()` method (but it still implements the Iterator trait).
