This is a Typst plugin written in Rust that parses VHDL files and returns some information about them in Typst records.

The source code is in the src/ directory:
- lib.rs: main library file, general utility functions
- parse_store.rs: VHDL file parsing, and storage for the parsed results, that can be recalled from any other function
- port_list.rs: Extracts the port list from the parsed VHDL
