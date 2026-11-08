# Rust Data Structures Best Practices — Research Documentation

**Project:** Database Storage Engine (Educational)  
**Date:** 2026-05-16  
**Status:** Complete research & recommendations

---

## Overview

This research documents best practices for implementing data structures in Rust, specifically tailored to your educational database internals project. The documentation covers library structure patterns, testing strategies, memory/concurrency safety, documentation conventions, and ecosystem recommendations.

---

## Document Guide

### 1. **QUICK_START_GUIDE.md** ← START HERE
**Purpose:** TL;DR version with immediate action items  
**Contains:**
- Core decisions at a glance
- Starter Cargo.toml
- Copy-paste file templates
- Testing checklist
- Quick reference table

**Best for:** Getting started immediately with concrete templates

---

### 2. **rust-data-structures-best-practices.md**
**Purpose:** Comprehensive reference on patterns and practices  
**Contains:**
- Library structure patterns (module organization, trait design)
- Testing patterns (unit, property-based, integration, doc tests)
- Memory/concurrency safety patterns (Arc/Mutex, unsafe boundaries, memory layout)
- Documentation standards (docs.rs conventions, doc tests, examples)
- Dependency ecosystem (recommended crates, what to avoid)

**Best for:** Understanding the "why" behind decisions; deep reference material

---

### 3. **architecture-recommendations.md**
**Purpose:** Concrete architecture applied to your database engine  
**Contains:**
- Project file structure (recommended layout for your crate)
- Module-by-module design (storage, index, query, buffer, db)
- Trait-based extensibility patterns
- Phase-by-phase implementation plan
- Integration testing examples
- Documentation priorities

**Best for:** Setting up your actual project structure; concrete examples

---

### 4. **testing-safety-patterns.md**
**Purpose:** Deep dive on testing and safety for data structures  
**Contains:**
- Unit testing best practices with fixtures and helpers
- Property-based testing patterns (proptest examples for B-trees)
- Integration testing across modules
- Memory safety patterns (single-threaded, concurrent, unsafe boundaries)
- Error handling with thiserror
- Test coverage strategy

**Best for:** When writing tests and safety-critical code; detailed examples

---

## Key Recommendations for Your Project

### Structure
```
✓ Single crate (not multi-crate workspace)
✓ Modules: storage/, index/, query/, buffer/, error.rs
✓ Traits for extensibility: StorageBackend, Query, Executor
✓ Clear module boundaries via mod.rs re-exports
```

### Testing
```
✓ Unit tests in #[cfg(test)] mod tests blocks (same file)
✓ Property tests with proptest for B-tree invariants
✓ Integration tests in tests/ directory
✓ Doc tests for all public API examples
→ Target: >80% code coverage
```

### Safety & Concurrency
```
✓ Start single-threaded (Rust's borrowing handles safety)
✓ Use Result<T> for error handling (never unwrap in library)
✓ Add Arc<RwLock<T>> + parking_lot only when multi-threading needed
✓ Keep unsafe code isolated with clear documentation
```

### Documentation
```
✓ Crate-level docs in lib.rs (//!)
✓ Module docs in mod.rs (//!)
✓ Function docs with examples (///)
✓ Example files in examples/
✓ Run cargo doc --open to verify
```

### Dependencies
```
Required:
  - thiserror (error handling)

Development:
  - proptest (property-based testing)
  - criterion (benchmarking)
  - tempfile (test fixtures)

Avoid (for educational project):
  - tokio / async runtimes
  - Heavy ORM dependencies
  - Production database crates
```

---

## Implementation Roadmap

### Phase 1: Storage Layer (Week 1)
- [ ] Create project structure
- [ ] Implement StorageBackend trait
- [ ] Implement FileStorage + Page abstraction
- [ ] Add unit tests
- [ ] Test: Write/read pages, verify persistence

### Phase 2: B-Tree Indexing (Week 2)
- [ ] Implement BTree and Node structures
- [ ] Implement insert, get, remove operations
- [ ] Add property-based tests
- [ ] Implement validate() for invariants
- [ ] Test: Ordering, range queries, rebalancing

### Phase 3: Query Layer (Week 3)
- [ ] Implement Query parser (simple DSL)
- [ ] Implement Executor
- [ ] Connect to storage + index layers
- [ ] Add integration tests
- [ ] Test: End-to-end query execution

### Phase 4: Documentation & Examples
- [ ] Write doc comments for all public APIs
- [ ] Create example programs
- [ ] Write architecture guide
- [ ] Verify all doc tests pass

---

## Quick Decision Matrix

| Question | Answer | Reference |
|----------|--------|-----------|
| Should I use multiple crates? | No, single crate is cleaner | arch-recommendations.md |
| Where do I put tests? | `#[cfg(test)]` in same file | testing-safety-patterns.md |
| Should I use async/tokio? | No, keep single-threaded initially | rust-data-structures-best-practices.md |
| How do I test B-tree invariants? | Use proptest property tests | testing-safety-patterns.md |
| When should I add multi-threading? | Only when needed; use Arc<RwLock> | testing-safety-patterns.md |
| What error handling should I use? | thiserror + Result types | QUICK_START_GUIDE.md |
| How should I document code? | Doc comments + examples | rust-data-structures-best-practices.md |
| Should I use unsafe code? | Avoid initially; isolate if needed | testing-safety-patterns.md |

---

## Learning Path Through Documents

**If you have 15 minutes:**
→ Read QUICK_START_GUIDE.md

**If you have 1 hour:**
→ QUICK_START_GUIDE.md + rust-data-structures-best-practices.md sections 1, 4, 5

**If you have 2-3 hours:**
→ All four documents in order: Quick Start → Best Practices → Architecture → Testing/Safety

**For specific questions:**
- "How do I organize my crate?" → architecture-recommendations.md
- "How should I test B-trees?" → testing-safety-patterns.md (section 1-2)
- "What dependencies should I use?" → rust-data-structures-best-practices.md (section 5)
- "How do I document this?" → rust-data-structures-best-practices.md (section 4)
- "How do I make it thread-safe?" → testing-safety-patterns.md (section 2)

---

## Testing Strategy at a Glance

```
Unit Tests                    Property Tests              Integration Tests
┌─────────────────────┐      ┌─────────────────────┐     ┌──────────────────┐
│ In source files     │      │ In source files     │     │ In tests/ dir    │
│ #[cfg(test)]        │      │ #[cfg(test)]        │     │ Cross-module     │
│ Test individual ops │      │ Test invariants     │     │ Full workflows   │
│ Test edge cases     │      │ Random inputs       │     │ Persistence      │
│ Test error cases    │      │ Catch edge cases    │     │ Integration pts  │
└─────────────────────┘      └─────────────────────┘     └──────────────────┘
Examples:                     Examples:                   Examples:
- insert(1, "a")            - prop_maintains_order      - write/read page
- get returns None          - prop_roundtrip             - btree serialization
- delete removes key        - prop_invariants            - query end-to-end
```

---

## Key Patterns to Remember

### Pattern 1: Trait-Based Extensibility
```rust
pub trait StorageBackend {
    fn read_page(&self, id: PageId) -> Result<Page>;
}

// Multiple implementations: FileStorage, MemoryStorage (for tests)
```

### Pattern 2: Module Re-exports
```rust
// src/storage/mod.rs
pub use self::page::Page;
pub use self::file_storage::FileStorage;
mod page;
mod file_storage;
```

### Pattern 3: Result-Based Error Handling
```rust
pub type Result<T> = std::result::Result<T, StorageError>;

pub fn read_page(&self, id: PageId) -> Result<Page> {
    // Returns Result instead of panicking
}
```

### Pattern 4: Inline Unit Tests
```rust
impl MyStruct {
    pub fn operation(&self) { /* ... */ }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_operation() { /* ... */ }
}
```

### Pattern 5: Property-Based Testing
```rust
proptest! {
    #[test]
    fn prop_maintains_order(keys in arb_keys()) {
        // Test with many random inputs
    }
}
```

---

## Common Pitfalls to Avoid

1. ❌ Using `unwrap()` in library code → Use `?` operator and Result types
2. ❌ Panicking on invalid input → Return Err() instead
3. ❌ No tests at all → Even educational code needs tests
4. ❌ Testing only happy path → Test edge cases and errors too
5. ❌ Overly complex trait hierarchies → Keep traits simple and focused
6. ❌ Writing unsafe code without tests → All unsafe must be thoroughly tested
7. ❌ Ignoring compiler warnings → Trust Rust's safety system
8. ❌ No documentation → Code without docs isn't useful to future maintainers

---

## Success Metrics

By the end of implementation, your project should have:

- [ ] **Structure:** Clear module organization with trait boundaries
- [ ] **Testing:** >80% code coverage with unit + property + integration tests
- [ ] **Safety:** All code type-safe, no panics in library code
- [ ] **Documentation:** All public functions documented with examples
- [ ] **Examples:** 2-3 working example programs in examples/
- [ ] **Errors:** Proper error handling with Result types throughout
- [ ] **Clarity:** Code is readable and demonstrates concepts well

---

## Additional Resources

### Official Rust Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) — Unsafe code
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [The Edition Guide](https://doc.rust-lang.org/edition-guide/)

### Key Crates to Study
- `std::collections::BTreeMap` — Standard B-tree implementation
- `parking_lot` — High-performance synchronization
- `crossbeam` — Concurrency utilities
- `proptest` — Property-based testing
- `criterion` — Benchmarking

### Example Projects
- `sled` — Educational key-value store (real-world patterns)
- `xi-editor` — Complex Rust data structures
- Rustc source code — How Rust implements common types

### Documentation Tools
- `cargo doc --open` — Local documentation
- https://docs.rs — Published documentation
- `rustdoc` — Official documentation format

---

## Questions to Ask While Implementing

1. **Does this module have one clear responsibility?**
   → If not, split it further

2. **Can I test this without the rest of the system?**
   → If not, your boundaries are wrong

3. **Would I be comfortable sharing this code with others?**
   → If not, add documentation or tests

4. **Does my error handling explain what went wrong?**
   → If not, improve error messages

5. **Are my invariants tested?**
   → If not, add property tests

6. **Can I read this code 6 months from now?**
   → If not, add comments explaining why

---

## Next Steps

1. **Read QUICK_START_GUIDE.md** (15 min)
2. **Copy starter Cargo.toml and lib.rs** (5 min)
3. **Create project structure** (10 min)
4. **Implement error.rs and storage/mod.rs** (1 hour)
5. **Add tests for each module** (concurrent)
6. **Read architecture-recommendations.md** (for guidance on phases 2-3)
7. **Implement B-tree** (2-3 hours)
8. **Implement query layer** (1-2 hours)
9. **Write documentation and examples** (1 hour)

---

## Document Maintenance

These documents are accurate as of 2026-05-16 for:
- Rust edition 2021
- Latest stable Rust toolchain
- Current crate versions (proptest 1.4, criterion 0.5, etc.)

As crates update, verify versions in:
- crates.io (https://crates.io)
- docs.rs (https://docs.rs)
- Official GitHub repositories

---

**Happy coding! These patterns will serve you well both for this educational project and for future Rust development.**
