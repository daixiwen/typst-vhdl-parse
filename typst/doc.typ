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

== Comments <sec-comments>

When parsing VHDL, the module looks after comments to describe the objects it finds. It looks for 
both comments on the same line than the object, and for comments on the line before it, as shown
in the following examples:

```vhdl
-- this is a leading comment (before the object)
signal my_signal: std_logic;

signal my_other_signal: std_logic; -- this is a trailing comment (with the object)
```

If an object has both a leading and a trailing comment, the parser doesn't know which one to pick.
This is why there is a `comment-priority` parameter, which tells it which comment to pick when both
are detected. The #link(label("vhdl-parse-parse()"), raw("parse()")) function has the
#link(label("vhdl-parse-parse.comment-priority"), raw("comment-priority")) parameter to give the 
default value, but it can then be overriden in all other function calls if needed.

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

= Implementation details

This section is only relevant if you are interested in the inner workings of the package or if you 
want to make any changes.

== Architecture

The package is made of a lightweight Typst file calling a WASM plugin. the plugin itself is written 
in Rust and uses the #link("https://lib.rs/crates/vhdl_lang")[`vhdl_lang`] crate. The plugin and the
Typst file communicate parameters and return values using byte arrays, and the CBOR protocol is
used on both sides to transfer more complex structures.

There is a very basic `AGENTS.md` file in the repository. I used #link("https://opencode.ai")[OpenCode]
to kickstart the project because I couldn't find how to use the 
#link("https://lib.rs/crates/vhdl_lang")[`vhdl_lang`] crate by mself, and to write some tests. The rest
was written manually, because it is pretty basic code, walking through the structures returned
by #link("https://lib.rs/crates/vhdl_lang")[`vhdl_lang`] to find the relevant information and putting
it in structures which can be serialized for Typst. I found it easier to just write the code than to
try to det en LLM to put it exactly in the way I wanted to. Maybe I'm just bad at prompting.

== Caching

The plugin caches results from a VHDL parsing, because I thought it would be very inneficient to redo it
at each function call. Unfortunately it makes it difficult to make the plugin
#link("https://typst.app/docs/reference/foundations/plugin#purity")[pure] as required by Typst: a plugin
function call must not have any observable side effects on future plugin calls, and given the same 
arguments, it must always return the same value.

I have therefore implemented a cache system. The #link(label("vhdl-parse-parse()"), raw("parse()"))
function stores the results of the parsing in a static cache, using the 
#link("https://docs.rs/lazy_static/latest/lazy_static/")[lazystatic]. It returns an ID for typst, and
the other functions use that ID as a parameter, so that the parsed VHDL can be extractd again from
the cache. To respect the purity requirement, the other functions also send the VHDL file name and 
the contents with the ID. That way the plugin can parse the VHDL file again if needed, and
technically the function is not dependent on previous calls, but still benefits from a cache.