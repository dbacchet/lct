use clap::{Arg, App};
use lct::parser;

fn main() {
    let matches = App::new("LCov Tool")
        .version("0.2.0")
        .author("Davide Bacchet <davide.bacchet@gmail.com>")
        .about("Simple LCOV coverage file parser")
        .arg(Arg::with_name("file")
                 .short("f")
                 .long("file")
                 .takes_value(true)
                 .help("name of the LCOV coverage file"))
        .arg(Arg::with_name("print_levels")
                 .short("l")
                 .long("levels")
                 .takes_value(false)
                 .default_value("-1")
                 .help("number of levels to print in the generated report"))
        .get_matches();

    let file_name = matches.value_of("file").unwrap();
    println!("Extracting coverage information from: {}", file_name);

    let print_levels = matches.value_of("print_levels").unwrap();
    let print_levels = match print_levels.parse::<i32>() {
                Ok(n) => n,
                Err(_) => -1
    };

    // parse the file and generate a tree with the coverage data
    let tree = parser::parse_coverage_file(file_name);
    // show coverage info
    tree.print_tree(print_levels);
}

