# rank-eval Code Updates

Updates needed to align with the new GitHub description: "Ranking evaluation metrics: NDCG, MAP, MRR, precision, recall. TREC format support."

## Files to Update

### 1. Cargo.toml

**Line 26:**
```toml
description = "Ranking evaluation metrics: NDCG, MAP, MRR, precision, recall. TREC format support."
```

### 2. src/lib.rs

**Lines 1-3:**
```rust
//! Ranking evaluation metrics: NDCG, MAP, MRR, precision, recall. TREC format support.
//!
//! This crate provides:
```

**Lines 4-5:**
```rust
//! - **Ranking evaluation metrics**: NDCG, MAP, MRR, Precision@K, Recall@K for binary and graded relevance
//! - **TREC format support**: Load and parse TREC run files and qrels
```

**Lines 8-40:** Reorder Quick Start examples:
- Move "Binary Relevance Metrics" section first (currently lines 24-33)
- Move "Graded Relevance Metrics" section second (currently lines 35-48)
- Move "TREC Format Parsing" section last (currently lines 8-22)

### 3. README.md

**Line 8:**
```markdown
Ranking evaluation metrics: NDCG, MAP, MRR, precision, recall. TREC format support.
```

**Lines 12-14:** Update "Why rank-eval?" section:
- Change "IR metrics" to "ranking metrics"
- Change "TREC format parsing is duplicated" to "evaluation code is duplicated"
- Emphasize metrics standardization first

**Lines 18-23:** Reorder Features section:
- Move "Ranking Evaluation Metrics" first
- Move "TREC Format Support" second

**Lines 168-173:** Reorder Modules section:
- Move `binary` first
- Move `graded` second
- Move `trec` third

## Summary

All changes emphasize that this is a **general-purpose ranking evaluation metrics library** with TREC format as a supporting feature, not the primary focus.

