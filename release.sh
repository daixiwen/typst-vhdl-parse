#!/bin/sh

# Compile a release and create the release folder
rm -rf release
mkdir -p release/vhdl-parse

cargo build --release
cp target/wasm32-unknown-unknown/release/typst_vhdl_parse.wasm release/vhdl-parse
cp LICENSE release/vhdl-parse
cp -r typst/doc typst/README.md typst/typst.toml typst/vhdl_parse.typ release/vhdl-parse
cp -r test release

typst compile --root release release/vhdl-parse/doc/doc.typ release/vhdl-parse/doc/doc.pdf
rm -r release/vhdl-parse/doc/doc.typ release/test

cd release; tar czf vhdl-parse-$1.tgz vhdl-parse
