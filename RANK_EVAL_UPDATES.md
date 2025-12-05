# rank-eval Code Updates Needed

Based on the updated GitHub description: "Ranking evaluation metrics: NDCG, MAP, MRR, precision, recall. TREC format support."

## Files to Update

### 1. Cargo.toml
**Current:**
```toml
description = "IR evaluation metrics and TREC format parsing for Rust"
```

**Should be:**
```toml
description = "Ranking evaluation metrics: NDCG, MAP, MRR, precision, recall. TREC format support."
```

### 2. src/lib.rs (crate-level doc comment)
**Current:**
```rust
//! IR evaluation metrics and TREC format parsing for Rust.
//!
//! This crate provides:
//! - **TREC format parsing**: Load and parse TREC run files and qrels
//! - **Binary relevance metrics**: NDCG, MAP, MRR, Precision@K, Recall@K for binary relevance
//! - **Graded relevance metrics**: NDCG and MAP for graded relevance judgments
```

**Should be:**
```rust
//! Ranking evaluation metrics: NDCG, MAP, MRR, precision, recall. TREC format support.
//!
//! This crate provides:
//! - **Ranking evaluation metrics**: NDCG, MAP, MRR, Precision@K, Recall@K for binary and graded relevance
//! - **TREC format support**: Load and parse TREC run files and qrels
```

### 3. README.md
**Current opening:**
```
IR evaluation metrics and TREC format parsing for Rust.
```

**Should be:**
```
Ranking evaluation metrics: NDCG, MAP, MRR, precision, recall. TREC format support.
```

**Current "Why rank-eval?" section:**
- Mentions "TREC format parsing is duplicated across projects" as a primary problem
- Should emphasize metrics standardization first, TREC format as secondary

**Current "Features" section:**
- Lists "TREC Format Parsing" first
- Should list metrics first, TREC format second

**Current "Quick Start" section:**
- Starts with "TREC Format Parsing" example
- Should start with metrics examples, TREC format examples second

## Summary

The main changes needed:
1. Update Cargo.toml description to match GitHub
2. Reorder lib.rs doc comment to emphasize metrics first
3. Reorder README sections to emphasize metrics first, TREC format second
4. Update README opening line to match new description

