use typst_vhdl_parse::{parse_store, port_list};

fn main()
{
    let contents = std::fs::read_to_string("test/adc_read.vhd").expect("couldn't read file");

    let (file_id, diags) = parse_store::parse("adc_read.vhd", "08", &contents).expect("error reading file");

    println!("File read! Diagnostics:");
    println!("*****");
    for diag in diags {
        println!("{diag}");
    }
   println!("*****   file ID: {file_id}");

   let design = parse_store::get_parsed(file_id).expect("parsed file not found");
   let ports = port_list::get_port_list(design).expect("port list not found");

   println!("Port list:");
   for port in ports {
    println!("{} ({}) {} {}", port.name, port.mode, port.port_type, port.constraint)
   }
}