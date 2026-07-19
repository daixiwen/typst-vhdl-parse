#import "vhdl_parse.typ" as vhdl-parse;
#import "@preview/diagraph:0.3.7"
#import "@preview/elembic:1.1.0"

= VHDL Parsing

#let parsed_file = vhdl-parse.parse("test.vhd", read("../test/test.vhd"))

Messages from the parser:

#for message in parsed_file.messages {
  [ - #message ]
}

= Port list

#let ports = vhdl-parse.port-list(parsed_file)

#table(
    columns: (auto, auto, auto, 1fr),
    table.header([name], [mode], [type], [description]),
    ..for entry in ports {
        ( [#entry.name], [#entry.mode], [#{entry.port-type}#{entry.constraint}], [#entry.description])
    }
)

= Generics list

#let generics = vhdl-parse.generic-list(parsed_file)

#table(
    columns: (auto, auto, auto, 1fr),
    table.header([name],[type], [description], [default value]),
    ..for entry in generics {
        ( [#entry.name], [#{entry.generic-type}#{entry.constraint}], [#entry.description], [#entry.default-value])
    }
)

#let generic-elem = elembic.element.declare(
  "generic",
  prefix: "eidel/fpga",
  doc: "indicates a generic has been described",
  display: it => raw(it.arg),
  fields: (
    elembic.field("arg", str, required: true),  // typed field for your argument
  ),
)


Here I am describing the #generic-elem("flag") generic and here the #generic-elem("value") generic.

#let check-generics(generic_list) = {
    context{
        let described_generic_list = elembic.query(generic-elem).map(it => elembic.fields(it).arg)
        let detected_generic_list = generic_list.map(it => it.name)

        for detected_generic in detected_generic_list {
            if not described_generic_list.contains(detected_generic) {
                text(fill:red, [ missing description for generic #raw(detected_generic)
                
                 ])
            }
        }
        for described_generic in described_generic_list {
            if not detected_generic_list.contains(described_generic) {
                text(fill:red, [ description for non existing generic #raw(described_generic)
                
                 ])
            }
        }
    }
}

#check-generics(generics)

= FSM 

#let fsm = vhdl-parse.fsm(parsed_file,"fsm.state")

List of states:

#table(
    columns: (auto, 1fr),
    table.header([state], [description]),
    ..for state in fsm.states {
        ( [#state.name], [#state.description])
    }
)

Diagram:

#let dotfile = vhdl-parse.fsm-dot(
    parsed_file, "fsm.state", 
    font-name: "DejaVu Sans",
    state-shape: "septagon",
    state-background-color: gray.lighten(50%),
    state-line-color: green.darken(50%),
    state-text-color: olive,
    state-font-size: 12,
    default-state-shape: "box",
    default-state-background-color: red.lighten(50%),
    default-state-line-color: red,
    default-state-text-color: red.darken(50%),
    default-state-font-size: 16,
    transition-line-color: blue.darken(30%),
    transition-text-color: blue.darken(50%),
    transition-font-size: 8,
    )

#diagraph.render(dotfile)

//#dotfile

= Instances

#let instances = vhdl-parse.instanceslist(parsed_file)

#instances

= Constants

#let constants = vhdl-parse.constantslist(parsed_file)

#table(
    columns: (auto, auto, auto, 1fr),
    table.header([name], [type], [description], [value]),
    ..for entry in constants {
        ( [#entry.name], [#{entry.object_type}#{entry.constraint}], [#entry.description], [#entry.expression])
    }
)

= Signals

#let signals = vhdl-parse.signalslist(parsed_file)

#table(
    columns: (auto, auto, auto, 1fr),
    table.header([name], [type], [description], [initial value]),
    ..for entry in signals {
        ( [#entry.name], [#{entry.object_type}#{entry.constraint}], [#entry.description], [#entry.expression])
    }
)

= Types

#let types = vhdl-parse.typeslist(parsed_file)

#table(
    columns: (auto, auto, 1fr),
    table.header([name], [kind], [description]),
    ..for entry in types {
        ( [#entry.name], [#entry.kind], [#entry.description] )
    }
)
