LCov Tool
=========

Very simple command line tool that parses a LCOV report and print coverage statistics as a tree.

Usage
-----

```
lct --file path/to/lcov_coverage_report_file.dat --levels 3
```
will parse the file and print a report with the first 3 levels in the file/folder hierarchy.

Call `lct --help` for the full set of available options.

Build
-----

Install [Rust](https://www.rust-lang.org) and:
```
cargo build --release
```

the executable will be in `target/release/lct`
