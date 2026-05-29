//! Per-request arena allocator (L-01 mitigation).
//!
//! All parse tree nodes, zone structs, and comment anchor maps produced during
//! a single `format()` call are allocated in this arena. The arena is dropped
//! at the end of the call, freeing everything in a single deallocation.
//!
//! This eliminates heap fragmentation from the repeated alloc/free of individual
//! CST nodes across many format operations. In WASM's linear memory model,
//! fragmentation is permanent — there is no compacting GC.

use bumpalo::Bump;

/// A per-request bump allocator.
///
/// Create one at the top of `format()` and drop it at the end.
/// Do not store `RequestArena` across multiple `format()` calls.
pub struct RequestArena {
    bump: Bump,
}

impl RequestArena {
    /// Allocate a new arena with an initial capacity of 64KB.
    ///
    /// The bump allocator grows automatically if more space is needed.
    /// The 64KB initial size covers small-to-medium source files without
    /// any reallocation.
    pub fn new() -> Self {
        RequestArena {
            bump: Bump::with_capacity(64 * 1024),
        }
    }

    /// Allocate a value in this arena.
    ///
    /// The returned reference is valid for the lifetime of the arena.
    /// All arena-allocated values are freed when the arena is dropped.
    #[allow(dead_code)]
    pub fn alloc<T>(&self, val: T) -> &T {
        self.bump.alloc(val)
    }

    /// Allocate a byte slice by copying `src` into the arena.
    #[allow(dead_code)]
    pub fn alloc_slice_copy(&self, src: &[u8]) -> &[u8] {
        self.bump.alloc_slice_copy(src)
    }

    /// Return the number of bytes currently allocated in this arena.
    #[allow(dead_code)]
    pub fn allocated_bytes(&self) -> usize {
        self.bump.allocated_bytes()
    }
}

impl Default for RequestArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_alloc_and_read() {
        let arena = RequestArena::new();
        let val = arena.alloc(42u32);
        assert_eq!(*val, 42);
    }

    #[test]
    fn arena_slice_copy() {
        let arena = RequestArena::new();
        let src = b"hello world";
        let copy = arena.alloc_slice_copy(src);
        assert_eq!(copy, src);
    }

    #[test]
    fn arena_reports_allocated_bytes() {
        let arena = RequestArena::new();
        arena.alloc(0u64);
        assert!(arena.allocated_bytes() >= 8);
    }
}
