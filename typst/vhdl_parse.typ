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
/// - parsed_file (struct):           The parsed filed object, as returned by parse() 
/// - read_variable_name (string):    The name of the signal or variable holding the current fsm state
/// - write_variable_name (string):   (optional) The name of the signal or variable holding the next fsm state, if different from read_variable_name 
/// - comment_priority (string):      (optional) override the default comment priority, either "leading" or "trailing" 
/// -> a string with the DOT description 
#let fsm_dot(parsed_file, read_variable_name, write_variable_name: none, comment_priority : none) = {
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
  return cbor(parsed_file.plugin.get_fsm_as_dot(bytes(parsed_file.id), cbor.encode(fsmconfig)))
}
