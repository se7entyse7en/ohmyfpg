# HISTORY

## Unreleased

### Added

- Added comparison against `psycopg`
- Added initial support to extended query with binary format

### Changed

- Reduced `malloc`s by ~55% for `simple_query` example
- [internal] Replace `Vec<u8>` with `bytes` in the hottest places

## [v0.3.0 - 2022-09-29](https://github.com/se7entyse7en/ohmyfpg/compare/v0.2.1...v0.3.0)

### Changed

- Improved performance and added proper comparison against `asyncpg`

## [v0.2.1 - 2022-09-11](https://github.com/se7entyse7en/ohmyfpg/compare/v0.2.0...v0.2.1)

### Fixed

- Fix version bump of the `Cargo.lock`s files

## [v0.2.0 - 2022-09-11](https://github.com/se7entyse7en/ohmyfpg/compare/v0.1.0...v0.2.0) [YANKED]

### Added

- Add script to compare performance with `asyncpg`
- [internal] Add `RawBackendMessage` in order to be able to split backend message identification from the full payload parsing
- [internal] Add `Framer` that handles eager reading from tcp stream concurrently

### Changed

- Renamed `PyInvalidDSNError` -> `PyInvalidDsnError`
- Improved performance by reading buffer eagerly instead of two syscalls (header + body) per message
- [internal] Split rust part into two workspaces (core + binding) to ease benchmarking and profiling of core

## [v0.1.0 - 2022-08-25](https://github.com/se7entyse7en/ohmyfpg/compare/v0.0.0...v0.1.0)

### Added

- Added a `connect` function returning a `Connection` object
- Implement a `fetch` method returning columnar data as `numpy` arrays with support to numerical Postgres types (`int2`, `int4`, `int8`, `float4`, `float8`)

## [v0.0.0 - 2022-06-26](https://github.com/se7entyse7en/ohmyfpg/compare/95f47c4cee38fad74a969ec34e5169c6e4e23c38...v0.0.0)

- Project inception! :tada:
