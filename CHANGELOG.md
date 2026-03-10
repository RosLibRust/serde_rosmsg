# Changelog for roslibrust_serde_rosmsg

## Unreleased

### Added

### Changed

### Fixed

## 0.6.0

### Added

### Changed

- Fixed dependency versions to fix build with minimal versions

### Fixed

## 0.5.1

### Added

- Implemented `is_human_readable()` to return `false` for both Serializer and Deserializer, correctly indicating that ROSMSG is a binary format

### Changed

- Updated `byteorder` dependency from 1.0.0 to 1.5

### Fixed

- Suppressed `unexpected_cfgs` warning from error-chain crate

## 0.5.0

### Added

### Changed

### Fixed

- Improved serialization performance with to_vec by ~95% on a 1080p color image
