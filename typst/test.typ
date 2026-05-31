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

#let dotfile = vhdl_parse.fsm_dot(
    parsed_file, "fsm", 
    font_name: "DejaVu Sans",
    state_shape: "septagon",
    state_background_color: gray.lighten(50%),
    state_line_color: green.darken(50%),
    state_text_color: olive,
    state_font_size: 12,
    default_state_shape: "box",
    default_state_background_color: red.lighten(50%),
    default_state_line_color: red,
    default_state_text_color: red.darken(50%),
    default_state_font_size: 16,
    )

#diagraph.render(dotfile)

//#dotfile

