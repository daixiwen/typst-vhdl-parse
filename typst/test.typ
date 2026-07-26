#import "vhdl_parse.typ" as vhdl-parse;

= Tests:

== VHDL Parsing

#let parsed-file = vhdl-parse.parse("test.vhd", read("../test/test.vhd"))

#assert(parsed-file.messages.len() == 0, message: "errors when parsing")

== Port list

#let ports = vhdl-parse.port-list(parsed-file)

#assert(
  ports == (
    (
      name: "clock",
      mode: "in",
      port-type: "std_logic",
      constraint: none,
      description: "main clock",
      default-value: none,
    ),
    (
      name: "sreset",
      mode: "in",
      port-type: "std_logic",
      constraint: none,
      description: "main reset, synchronous, active high",
      default-value: "'0'",
    ),
    (
      name: "output_a",
      mode: "out",
      port-type: "std_logic",
      constraint: none,
      description: "a regular output",
      default-value: none,
    ),
    (
      name: "input_b",
      mode: "in",
      port-type: "std_logic",
      constraint: none,
      description: "one input",
      default-value: none,
    ),
    (
      name: "output_c",
      mode: "out",
      port-type: "std_logic",
      constraint: none,
      description: "another output",
      default-value: none,
    ),
    (
      name: "input_d",
      mode: "in",
      port-type: "std_logic",
      constraint: none,
      description: "another input",
      default-value: none,
    ),
    (
      name: "data_in",
      mode: "in",
      port-type: "std_logic_vector",
      constraint: "(15 downto 0)",
      description: none,
      default-value: none,
    ),
    (
      name: "data_out",
      mode: "out",
      port-type: "std_logic_vector",
      constraint: "(15 downto 0)",
      description: "data out",
      default-value: none,
    ),
  ),
  message: "wrong ports list",
)

== Generics list

#let generics = vhdl-parse.generic-list(parsed-file)

#assert(
  generics == (
    (
      name: "flag",
      generic-type: "boolean",
      constraint: none,
      description: "a flag, can be true or false",
      default-value: none,
    ),
    (
      name: "value",
      generic-type: "std_logic_vector",
      constraint: "(15 downto 0)",
      description: "a value with a default",
      default-value: "x\"DEAD\"",
    ),
  ),
  message: "wrong generics list",
)

== FSM 

#let fsm = vhdl-parse.fsm(parsed-file,"fsm_state")

#assert(
  fsm == (
    default_state: "reset",
    states: (
      (
        name: "reset",
        description: "start and reset state",
        transitions: (
          (
            destination: "idle",
            condition: "",
            description: "out of reset",
          ),
        ),
      ),
      (
        name: "idle",
        description: "normal state when nothing happens",
        transitions: (
          (
            destination: "read_input",
            condition: "input_b = '1'",
            description: "new input",
          ),
        ),
      ),
      (
        name: "read_input",
        description: "waiting for input",
        transitions: (
          (
            destination: "idle",
            condition: "input_d = '1'",
            description: none,
          ),
          (
            destination: "write_output",
            condition: "data_in = std_logic_vector(value)",
            description: "correct input",
          ),
        ),
      ),
      (
        name: "write_output",
        description: "send output",
        transitions: (
          (
            destination: "idle",
            condition: "input_b = '0'",
            description: none,
          ),
          (
            destination: "write_output",
            condition: "",
            description: "stay",
          ),
        ),
      ),
    ),
  ),
  message: "wrong fsm",
)

#let dotfile = vhdl-parse.fsm-dot(
    parsed-file, "fsm_state", 
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

// this currently doesn't work in some cases, like in Github CI. I need to find a better way to test this
//#assert(dotfile == "digraph {
//  fontname=\"DejaVu Sans\"
//  node [fontname=\"DejaVu Sans\", shape=septagon, style=filled, fillcolor=\"#d5d5d5\", color=\"#176620\", fontcolor=\"#3d9970\", fontsize=12]
//  edge [fontname=\"DejaVu Sans\", color=\"#005198\", fontcolor=\"#003a6c\", fontsize=8]
//  reset [shape=box, fillcolor=\"#ffa09b\", color=\"#ff4136\", fontcolor=\"#80211b\", fontsize=16]
//  reset -> idle[label=\"out of reset\"]
//  idle -> read_input[label=\"new input\"]
//  read_input -> idle[label=\"input_d = '1'\"]
//  read_input -> write_output[label=\"correct input\"]
//  write_output -> idle[label=\"input_b = '0'\"]
//  write_output -> write_output[label=\"stay\"]
//}
//", message: "wrong dot file")

== Instances

#let instances = vhdl-parse.instances-list(parsed-file)

#assert(
  instances == (
    (
      label: "my_instance",
      entity: "work.comp",
      description: "an example instantiation",
      generics_map: ((origin: "enabled", expression: "true"),),
      ports_map: (
        (origin: "clk", expression: "clock"),
        (origin: "rst", expression: "reset"),
        (origin: "input", expression: "input_b"),
        (origin: "output", expression: "output_a"),
      ),
    ),
  ),
  message: "wrong instances",
)

== Constants

#let constants = vhdl-parse.constants-list(parsed-file)

#assert(
  constants == (
    (
      name: "c_ones",
      object_type: "std_logic_vector",
      constraint: "(31 downto 0)",
      expression: "(others => '1')",
      description: "a constant",
    ),
  ),
  message: "wrong constants",
)

== Signals

#let signals = vhdl-parse.signals-list(parsed-file)

#assert(
  signals == (
    (
      name: "fsm",
      object_type: "fsm_wrapper_t",
      constraint: none,
      expression: none,
      description: "state machine signal",
    ),
    (
      name: "mysignal",
      object_type: "unsigned",
      constraint: "(15 downto 0)",
      expression: "(others => '0')",
      description: "a signal with a comment on the same line",
    ),
  ),
  message: "wrong signals",
)

== Types

#let types = vhdl-parse.types-list(parsed-file)

#assert(
types == (
    (
      name: "fsm_t",
      kind: "enumeration",
      definition: (
        Enumeration: (
          (
            element_name: "reset",
            description: "reset, initial state",
          ),
          (
            element_name: "idle",
            description: "state when not doing anything",
          ),
          (
            element_name: "read_input",
            description: "check what's on the inputs",
          ),
          (
            element_name: "write_output",
            description: "do something on the outputs",
          ),
        ),
      ),
      description: "enumeration type for the FSM",
    ),
    (
      name: "fsm_wrapper_t",
      kind: "record",
      definition: (
        Record: (
          (
            element_name: "state",
            element_type: "fsm_t",
            description: "the actual fsm type",
          ),
        ),
      ),
      description: "a wrapper around the fsm type to test record element access in FSM detection",
    ),
    (
      name: "byte_t",
      kind: "subtype",
      definition: (
        SubType: (
          subtype: "std_logic_vector",
          constraint: "(7 downto 0)",
        ),
      ),
      description: "a subtype",
    ),
    (
      name: "matrix_t",
      kind: "array",
      definition: (
        Array: (range: "31 downto 0,7 downto 0", subtype: "byte_t"),
      ),
      description: "an array",
    ),
  ),
  message: "wrong types",
)

= All tests OK!!!
