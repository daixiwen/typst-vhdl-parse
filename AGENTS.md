This is a Typst plugin written in Rust that parses VHDL files and returns some information about them in Typst records.

The source code is in the src/ directory:
- lib.rs: main library file, general utility functions
- parse_store.rs: VHDL file parsing, and storage for the parsed results, that can be recalled from any other function
- *.rs: Extracts the information  from the parsed VHDL

In the typst/ directory, there is a vhdl_parse.typ file which is the Typst API file, that calls the WASM plugin

To run the tests on linux:
cargo test --target x86_64-unknown-linux-gnu
