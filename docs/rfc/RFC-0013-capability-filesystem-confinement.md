# RFC-0013: Capability-based filesystem confinement

## Title

Capability-based filesystem confinement for `ag-storage`

## Motivation

The native storage backend must guarantee that object operations cannot escape
the configured root, including when an attacker can replace path components
with symlinks between validation and I/O.

## Problem

Path canonicalization followed by a separate open, create, or rename operation
is subject to time-of-check/time-of-use races. It also cannot validate a final
path that does not exist without walking its existing ancestors.

## Alternatives

- Re-check each path component with `symlink_metadata`: rejected because the
  later I/O still races with component replacement.
- Use platform-specific `openat2`/handle APIs directly: rejected because it
  requires duplicated unsafe and operating-system-specific code.
- Keep canonicalization and document the limitation: rejected because native
  storage is a security boundary.

## Design

Add `cap-std` and `cap-fs-ext` and open the configured root once as a `cap_std::fs::Dir`.
Native reads, writes, deletes, existence checks, renames, and directory walks
walk each parent directory without following symlinks and operate relative to the resulting capability. Temporary files and their final rename
remain inside the same directory capability.

Blocking filesystem operations run through Tokio's blocking pool so async
workers are not blocked.

## Risks

- One additional maintained dependency and its transitive platform support.
- Capability filesystem behavior must remain covered on Unix and Windows CI.

## Impact

The public API and object-key format remain unchanged. Attempts to traverse a
symlink outside the root fail as storage I/O errors and cannot access the target.

## Rollback

Remove `cap-std` and restore path-based operations only if an equivalent,
