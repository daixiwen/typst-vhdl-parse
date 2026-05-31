#import "vhdl_parse.typ" as vhdl_parse;
#import "@preview/diagraph:0.3.7"

= VHDL Parsing

#let parsed_file = vhdl_parse.parse("test.vhd", read("../test/test.vhd"))

Messages from the parser:

#for message in parsed_file.messages {
  [ - #message ]
}

= Port list

#let ports = vhdl_parse.portlist(parsed_file)

#table(
    columns: (auto, auto, auto, 1fr),
    table.header([name], [mode], [type], [description]),
    ..for entry in ports {
        ( [#entry.name], [#entry.mode], [#{entry.port_type}#{entry.constraint}], [#entry.description])
    }
)

= FSM 

#let fsm = vhdl_parse.fsm(parsed_file,"fsm")

List of states:

#table(
    columns: (auto, 1fr),
    table.header([state], [description]),
    ..for state in fsm.states {
        ( [#state.name], [#state.description])
    }
)

Diagram:

#diagraph.render(vhdl_parse.fsm_dot(parsed_file, "fsm"))
