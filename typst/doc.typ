#import "@preview/tidy:0.4.3" 
#import "@preview/codly:1.3.0": codly-init, no-codly, codly

#import "vhdl_parse.typ" as vhdl-parse

#show: codly-init.with()
#show raw.where(block: true): set text(size: .9em)

// utility function to create a table describing the dictionaries
#let dictionary-description(elements) = {
    table(
        columns: (auto, auto, 1fr),
        table.header([name], [type], [description]),
        ..for element in elements {
            (
                raw(element.at(0)),
                raw(element.at(1)),
                element.at(2)
            )
        }
    )
}

// parse the VHDL file here and give it to the examples
#let parsed-file = vhdl-parse.parse("test.vhd", read("../test/test.vhd"))

#let docs = tidy.parse-module(read("vhdl_parse.typ"), scope: (vhdl-parse: vhdl-parse, parsed-file: parsed-file, dictionary-description: dictionary-description))

#tidy.show-module(docs, show-outline: false, sort-functions: none)
