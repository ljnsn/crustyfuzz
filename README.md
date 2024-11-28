# crustyfuzz

A string matching library for Python written in Rust. It uses the string similarity calculations from [`RapidFuzz`](https://github.com/rapidfuzz/RapidFuzz). However, there are a couple of aspects that set `CrustyFuzz` apart from `RapidFuzz`:

1. It's cooler, because it's written in Rust
2. I'm kidding, there are probably no benefits to using this over `RapidFuzz`, I wrote it for fun and to improve my Rust skills (although I will benchmark it eventually).

## Roadmap

The roadmap to feature parity.

- [x] Simple Ratio
- [x] Partial Ratio
- [x] Token Sort Ratio
- [x] Token Set Ratio
- [x] Weighted Ratio
- [x] Quick Ratio
- [x] Process
