LCov Tool
=========

Very simple command line tool that parses a LCOV report and print coverage statistics as a tree.

Usage
-----

```
lct path/to/lcov_coverage_report_file.dat
```

Build
-----

Install [Rust](https://www.rust-lang.org) and:
```
cargo build --release
```

the executable will be in `target/release/lct`
