# topo

The [topo](https://docs.rs/topo) crate provides incremental caching and identifiers for
repeated function invocations. Together with a change notification mechanism it can be used
to implement a form of [incremental computing](https://en.wikipedia.org/wiki/Incremental_computing).

<!-- categories: Added, Removed, Changed, Deprecated, Fixed, Security -->

## [0.10.0-pre] - unreleased

### Fixed

- `Id` generation is no longer vulnerable to hashing collisions.

### Added

- #[nested] allows specifying a `slot`.
- `Cache`, `GlobalCache` types for storing interned and memoized values.
- `CacheHandle`, `GlobalCacheHandle` types for multiple-owner access to caches, implementing
  `memo_with` with careful locking to allow nested calls in the future.
- `root` free function for allowing one to re-root a call topology (i.e. if running inside of a
  broader one).

### Removed

- `Point` is no longer `pub`.
- `#![feature(track_caller)]` is no longer needed.

### Changed

- `illicit` dependency updated to 1.0.
- `impl Trait` has been removed from public APIs where it may cause accidental `Send`/`!Send`
  contracts.

## [0.9.4] - 2019-12-26

### Changed

- Updated `illicit` dependency to `0.9.0`.

## [0.9.3] - 2019-12-25

### Changed

- `#[track_caller]` is used to generate `Id`s, replacing macros.
- Use `DefaultHasher` instead of `FnvHasher`.

### Added

- `call`, `call_in_slot` functions.

### Removed

- `call!` and `unstable_make_topo_macro!` macros.

## [0.9.2] - 2019-11-23

### Changed

- Using `fnv` crate for hashing `Id`s.

## [0.9.1] - 2019-11-21

### Removed

- `#![warn(intra_doc_resolution_failure)]` was causing docs.rs issues due to root_html_url.

## [0.9.0] - 2019-11-19

### Added

- `#![forbid(unsafe_code)]`
- `call!` accepts a "slot" other than the number of times a callsite has been seen. The callsite
  count is still the default.
- Invoking `call!` when no `Point` has already been entered will now create a new root and enter it
  before executing the block.

### Changed

- Rename `#[bound]` to `#[nested]`.
- Rename `current_callsite_count` to `Callsite::current_count`.

### Removed

- `env!`, `Env`, `#[from_env]` moved to `illicit` crate.
- `root!` removed in favor of creating a new root whenever `call!` is invoked outside of a `Point`.

## [0.8.2] - 2019-08-20

### Fixed

- `root!` no longer hides the outer environment from the called block.

## [0.8.1] - 2019-08-17

### Changed

- `Id`'s `Debug` impl uses hex.

### Fixed

- Incorrect line endings.

## [0.8.0] - 2019-06-23

### Added

- `#[topo::bound]` attaches a function to the topology.
- `root!` and `call!` macros attach arbitrary blocks to a new or the current topology respectively,
  entering new `Point`s for each call, each of which has a (mostly) unique `Id`.
- `env!` macro allows declaring type-indexed implicit variables, produces `Env` instances.

## [0.1.0] - 2019-05-26

Published to reserve name on crates.io.