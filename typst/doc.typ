#import "@preview/tidy:0.4.3" 
#import "@preview/codly:1.3.0": codly-init, no-codly, codly

#import "vhdl_parse.typ" as vhdl-parse

#show: codly-init.with(
)
#show raw.where(block: true): set text(size: .9em)

// parse the VHDL file here and give it to the examples
#let parsed-file = vhdl-parse.parse("test.vhd", read("../test/test.vhd"))

#let docs = tidy.parse-module(read("vhdl_parse.typ"), scope: (vhdl-parse: vhdl-parse, parsed-file: parsed-file))

#tidy.show-module(docs, show-outline: false, sort-functions: none)
