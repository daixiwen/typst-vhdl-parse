#let plug = plugin("typst_vhdl_parse.wasm")

/// Parse a VHDL file
///
/// - file_name (string):                  The VHD file name
/// - contents (string or bytes):          The VHDL code 
/// - vhdl_variant (string or number):     (optional) the VHDL variant. Default: 2008
/// - comment_priority (string)            (optional) the default comment priority. Either "leading" or "trailing". Default "trailing"
/// 
/// -> a parsed file object (dictionary). The "messages" item is a list of warnings or errors from the parser
#let parse(file_name, contents, vhdl_variant : 2008, comment_priority : "trailing") = {
  // arguments chech and conversion
  assert(type(file_name) == str, message: "file_name must be a string")
  if type(contents) == str {
    contents = bytes(contents)
  }
  assert(type(contents) == bytes, message: "contents must be a string or bytes")
  vhdl_variant = str(vhdl_variant)
  assert(("93", "08", "19", "1993", "2008", "2019").contains(vhdl_variant), message: "unknown VHDL variant")
  assert(("trailing", "leading").contains(comment_priority), message: "comment priority must be \"trailing\" or \"leading\"")

  // parse the file and return the results
  let result = cbor(plug.parse(bytes(file_name), bytes(vhdl_variant), contents))
  return (
    "plugin":           plug,
    "id":               result.id,
    "messages":         result.messages,
    "comment_priority": comment_priority
  )
}

/// portlist: returns the portlist from the first entity found in the parsed file
///
/// - parsed_file (struct):      The parsed filed object, as returned by parse() 
/// - comment_priority (string): (optional) override the default comment priority, either "leading" or "trailing" 
/// 
/// -> an array of dictionaries, with in each item:
///    - name (string):                   the port name
///    - mode (string):                   the port mode (in, out, inout, buffer)
///    - port_type (string):              the type of the port
///    - constraint (string):             the type constraint, for example: "(15 downto 0)"
///    - description (string or none):    a comment describing the port
#let portlist(parsed_file, comment_priority : none) = {
  // arguments check and conversion
  assert(type(parsed_file) == dictionary, message: "parsed_file must be the return value from the parse() function")
  if comment_priority == none {
    comment_priority = parsed_file.comment_priority
  }
  assert(("trailing", "leading").contains(comment_priority), message: "comment priority must be \"trailing\" or \"leading\"")

  // call the plugin and return the results
  return cbor(parsed_file.plugin.get_port_list(bytes(parsed_file.id), bytes(comment_priority)))
}

/// genericlist: returns the generics list from the first entity found in the parsed file
///
/// - parsed_file (struct):      The parsed filed object, as returned by parse() 
/// - comment_priority (string): (optional) override the default comment priority, either "leading" or "trailing" 
/// 
/// -> an array of dictionaries, with in each item:
///    - name (string):                   the port name
///    - generic_type (string):           the type of the generic
///    - constraint (string or none):     the type constraint, for example: "(15 downto 0)"
///    - description (string or none):    a comment describing the port
///    - default_value (string or none):  the default contents of the generic
#let genericlist(parsed_file, comment_priority : none) = {
  // arguments check and conversion
  assert(type(parsed_file) == dictionary, message: "parsed_file must be the return value from the parse() function")
  if comment_priority == none {
    comment_priority = parsed_file.comment_priority
  }
  assert(("trailing", "leading").contains(comment_priority), message: "comment priority must be \"trailing\" or \"leading\"")

  // call the plugin and return the results
  return cbor(parsed_file.plugin.get_generic_list(bytes(parsed_file.id), bytes(comment_priority)))
}

/// fsm: returns information about a state machine found in the parsed file
///
/// - parsed_file (struct):           The parsed filed object, as returned by parse() 
/// - read_variable_name (string):    The name of the signal or variable holding the current fsm state
/// - write_variable_name (string):   (optional) The name of the signal or variable holding the next fsm state, if different from read_variable_name 
/// - comment_priority (string):      (optional) override the default comment priority, either "leading" or "trailing" 
///
/// -> a dictionary, with the following items:
/// - default_state (string):         the default state (i.e. found in an "others" case)
/// - states (array):                 an array of states, where each state is a dictionary with the following elements:
///   - name (string):                  the state name
///   - description (string or none):   the comment describing the state
///   - transition (array):             an array of transitions, where each transition is a diciotnary with the following elements:
///     - destination (string):           the state it is transitioned to
///     - consition (string)              the condition for the transition, if found
///     - description (string or none):   the comment describing the condition, if found
#let fsm(parsed_file, read_variable_name, write_variable_name: none, comment_priority : none) = {
  // arguments check and conversion
  assert(type(parsed_file) == dictionary, message: "parsed_file must be the return value from the parse() function")
  assert(type(read_variable_name) == str, message: "read_variable_name must be a string")
  if write_variable_name == none {
    write_variable_name = read_variable_name
  }
  assert(type(write_variable_name) == str, message: "write_variable_name must be a string")
  if comment_priority == none {
    comment_priority = parsed_file.comment_priority
  }
  assert(("trailing", "leading").contains(comment_priority), message: "comment priority must be \"trailing\" or \"leading\"")

  // build config structure
  let fsmconfig = (
    "read_variable_name": read_variable_name,
    "write_variable_name": write_variable_name,
    "comment_priority_trailing": (comment_priority == "trailing")
  )

  // call plugin
  return cbor(parsed_file.plugin.get_fsm_as_struct(bytes(parsed_file.id), cbor.encode(fsmconfig)))
}

/// fsm_dot: returns a description of an FDM in the DOT format, ready to be drawn by the diagraph package1
///
/// - parsed_file (struct):                    The parsed filed object, as returned by parse() 
/// - read_variable_name (string):             The name of the signal or variable holding the current fsm state
/// - write_variable_name (string):            (optional) the name of the signal or variable holding the next fsm state, if different from read_variable_name 
/// - comment_priority (string):               (optional) override the default comment priority, either "leading" or "trailing" 
/// - left_to_right(bool):                     (optional) if true, distribute the states left-to-right instead of top-to-bottom
/// - font_name(string):                       (optional) the font to use (has to be accessible to Typst)
/// - state_shape (string):                    (optional) the shape to use for states
/// - state_background_color (color):          (optional) the background color for states
/// - state_line_color (color):                (optional) the line color for states
/// - state_text_color (color):                (optional) the color of text for states
/// - state_font_size: (float):                (optional) the text size for states (in points)
/// - default_state_shape (string):            (optional) the shape to use for the default state
/// - default_state_background_color (string): (optional) the background color for the default state
/// - default_state_line_color (String):       (optional) the line color for the default state
/// - default_state_text_color (string):       (optional) the color of text for the default state
/// - default_state_font_size (float):         (optional) the text size for the default state (in points)
/// - transition_line_color (String):          (optional) the line color for the transitions
/// - transition_text_color (string):          (optional) the color of text for the transitions
/// - transition_font_size (float):            (optional) the text size for the transitions (in points)
/// 
/// -> a string with the DOT description 
#let fsm_dot(parsed_file, 
            read_variable_name, 
            write_variable_name: none, 
            comment_priority : none,
            left_to_right: false,
            font_name : "Helvetica,Arial,sans-serif",
            state_shape: "ellipse",                  
            state_background_color: white,       
            state_line_color: black,             
            state_text_color: black,             
            state_font_size: 14,             
            default_state_shape: "doublecircle",          
            default_state_background_color: none,
            default_state_line_color: none,     
            default_state_text_color: none,     
            default_state_font_size: none,                  
            transition_line_color: none,     
            transition_text_color: none,     
            transition_font_size: none                  
            ) = {
  // arguments check and conversion
  assert(type(parsed_file) == dictionary, message: "parsed_file must be the return value from the parse() function")
  assert(type(read_variable_name) == str, message: "read_variable_name must be a string")
  if write_variable_name == none {
    write_variable_name = read_variable_name
  }
  assert(type(write_variable_name) == str, message: "write_variable_name must be a string")
  if comment_priority == none {
    comment_priority = parsed_file.comment_priority
  }
  assert(("trailing", "leading").contains(comment_priority), message: "comment priority must be \"trailing\" or \"leading\"")
  let shapes = ( "box", "polygon", "ellipse", "oval", "circle", "point", "egg", "triangle", "plaintext", "plain", "diamond", "trapezium", "parallelogram", "house", "pentagon", "hexagon", "septagon", "octagon", "doublecircle", "doubleoctagon", "tripleoctagon", "invtriangle", "invtrapezium", "invhouse", "Mdiamond", "Msquare", "Mcircle", "rect", "rectangle", "square", "star", "none", "underline", "cylinder", "note", "tab", "folder", "box3d", "component", "promoter", "cds", "terminator", "utr", "primersite", "restrictionsite", "fivepoverhang", "threepoverhang", "noverhang", "assembly", "signature", "insulator", "ribosite", "rnastab", "proteasesite", "proteinstab", "rpromoter", "rarrow", "larrow", "lpromoter")
  assert(shapes.contains(state_shape), message: "state_shape needs to be a valid shape type. See https://graphviz.org/doc/info/shapes.html")
  assert(type(state_background_color) == color, message: "state_background_color needs to be a color")
  assert(type(state_line_color) == color, message: "state_line_color needs to be a color")
  assert(type(state_text_color) == color, message: "state_text_color needs to be a color")
  if type(state_font_size) == int {
    state_font_size = float(state_font_size)
  }
  assert(type(state_font_size) == float, message: "state_font_size needs to be a number")
  assert(shapes.contains(default_state_shape), message: "default_state_shape needs to be a valid shape type. See https://graphviz.org/doc/info/shapes.html")
  if default_state_background_color == none {
    default_state_background_color = state_background_color
  }
  assert(type(default_state_background_color) == color, message: "default_state_background_color needs to be a color")
  if default_state_line_color == none {
    default_state_line_color = state_line_color
  }
  assert(type(default_state_line_color) == color, message: "default_state_line_color needs to be a color")
  if default_state_text_color == none {
    default_state_text_color = state_text_color
  }
  assert(type(default_state_text_color) == color, message: "default_state_text_color needs to be a color")
  if default_state_font_size == none {
    default_state_font_size = state_font_size
  }
  if type(default_state_font_size) == int {
    default_state_font_size = float(default_state_font_size)
  }
  assert(type(default_state_font_size) == float, message: "default_state_font_size needs to be a number")
  if transition_line_color == none {
    transition_line_color = state_line_color
  }
  assert(type(transition_line_color) == color, message: "transition_line_color needs to be a color")
  if transition_text_color == none {
    transition_text_color = state_text_color
  }
  assert(type(transition_text_color) == color, message: "transition_text_color needs to be a color")
  if transition_font_size == none {
    transition_font_size = state_font_size
  }
  if type(transition_font_size) == int {
    transition_font_size = float(transition_font_size)
  }
  assert(type(transition_font_size) == float, message: "transition_font_size needs to be a number")

  // build config structures
  let fsmconfig = (
    "read_variable_name": read_variable_name,
    "write_variable_name": write_variable_name,
    "comment_priority_trailing": (comment_priority == "trailing")
  )
  let fsmdotconfig = (
    "left_to_right":                  left_to_right,
    "font_name":                      font_name,
    "state_shape":                    state_shape,
    "state_background_color":         state_background_color.to-hex(),
    "state_line_color":               state_line_color.to-hex(),
    "state_text_color":               state_text_color.to-hex(),
    "state_font_size":                state_font_size,
    "default_state_shape":            default_state_shape,
    "default_state_background_color": default_state_background_color.to-hex(),
    "default_state_line_color":       default_state_line_color.to-hex(),
    "default_state_text_color":       default_state_text_color.to-hex(),
    "default_state_font_size":        default_state_font_size,
    "transition_line_color":          transition_line_color.to-hex(),
    "transition_text_color":          transition_text_color.to-hex(),
    "transition_font_size":           transition_font_size,
  )

  // call plugin
  return cbor(parsed_file.plugin.get_fsm_as_dot(bytes(parsed_file.id), cbor.encode(fsmconfig), cbor.encode(fsmdotconfig)))
}
