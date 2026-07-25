#import "@preview/tidy:0.4.3" 
#import "@preview/codly:1.3.0": codly-init, no-codly, codly
#import "@preview/numbly:0.1.0": numbly

#import "vhdl_parse.typ" as vhdl-parse

// current version
#let version = "0.1.0"
#let release_date = "2026/07/25"

// document template
#show: codly-init.with()
#show raw.where(block: true): set text(size: .9em)
#set text(font: ("Helvetica","Arial","DejaVu Sans"))
#set heading(numbering:numbly(
    "{1}", "{1}.{2}", "{1}.{2}.{3}", none
))
#show link: it => {
    set text(fill: rgb("#7f007f")) 
    underline(it)
  }

// title page
#v(4em)
#align(center)[
    #text(weight:700, size: 2em, "Vhdl-parse")
    #v(1em)
    #text(size: 1.5em, "Documentation")
    #v(3em)
    Version #version, #release_date

    Sylvain Tertois

    #link("https://github.com/daixiwen/typst-vhdl-parse")

    #v(5em)
]

#text(weight: 700, size: 1.5em, "Summary")

`Vhdl-parse` is a typst module that parses VHDL files and extracts information from them that
can be presented in a typst document. It makes writing documentation for VHDL code easier and
let use the VHDL code as single source of truth.

#outline(depth: 2, title: "Table of contents")

// content
#show heading.where(level: 1): it => {
    pagebreak();
    it
}

= Usage

== Invocation

Import the `vhdl-parse` module as usual:

#raw(lang: "typ", block: true, "#import \"@preview/vhdl-parse:" + version)

Use the #link(label("vhdl-parse-parse()"), raw("parse()")) function to parse your VHDL file. 

```typ
#let parsed-file = vhdl-parse.parse(
    "test.vhd", 
    read("test.vhd"))

#assert(parsed-file.messages.len() == 0, message. "errors were found in the VHDL file")
```

The object returned is used as parameter for all the other functions to extract different kind
of information from the VHDL contents. The `message` field contains errors returned by the
VHDL parser.

Please note that the `vhdl-parse` module does not present the extracted data in any 
ready-to-display state. It returns arrays and dictionary with the requested data, and it
is up to the user to write a template to present the data in a pretty and consistent way
across the documentation.

Please refer to #lower[@sec-fref] for more details about all the functions that can be called
with the parsed file.

== Comments



= Functions reference <sec-fref>

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

#let docs = tidy.parse-module(read("vhdl_parse.typ"), name: "vhdl-parse", scope: (vhdl-parse: vhdl-parse, parsed-file: parsed-file, dictionary-description: dictionary-description))

#tidy.show-module(docs, show-outline: false, sort-functions: none, first-heading-level: 1, show-module-name: false)
