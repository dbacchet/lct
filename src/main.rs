use regex::Regex;
use clap::{Arg, App};

#[derive(Debug)]
struct EntryTree {
    name: String,
    lines_covered: u32,
    lines_instrumented: u32,
    expand: bool,
    children: Vec<EntryTree>,
}

impl EntryTree {
    // create a tree
    fn new(root_name: &str) -> EntryTree {
        EntryTree {
            name : String::from(root_name),
            lines_covered: 0,
            lines_instrumented: 0,
            expand : true,
            children : Vec::new(),
        }
    }

    // get or create an entry 
    fn get_or_create(&mut self, name: &str) -> &mut EntryTree {
        if self.name == name {
            return self
        }
        let pos = self.children.iter().position(|x| { x.name == name });
        match pos {
            Some(idx) => &mut self.children[idx],
            None => { self.children.push(EntryTree::new(name)); 
                      self.children.last_mut().unwrap() }
        }
    }

    fn get_or_create_with_path(&mut self, path: &str, lines_covered: u32, lines_instrumented: u32) -> &mut EntryTree {
        let tokens = path.split("/");
        let mut node = self;
        for t in tokens {
            node = node.get_or_create(t);
        }
        node.lines_covered = lines_covered;
        node.lines_instrumented = lines_instrumented;
        node
    }

    fn update_coverage_statistics(&mut self) {
        fn update_coverage_recursive(node: &mut EntryTree) -> (u32, u32) {
            if node.children.len()==0 {
                (node.lines_covered, node.lines_instrumented)
            } else {
                let mut children_covered = 0;
                let mut children_instrumented = 0;
                for c in node.children.iter_mut() {
                    let stats = update_coverage_recursive(c);
                    children_covered += stats.0;
                    children_instrumented += stats.1;
                }
                node.lines_covered = children_covered;
                node.lines_instrumented = children_instrumented;
                (children_covered, children_instrumented)
            }
        }
        update_coverage_recursive(self);
    }

    fn print_tree(&self, max_depth: i32) {
        fn print_recursive(node: &EntryTree, depth: i32, max_depth: i32) {
            if max_depth>0 && depth>max_depth {
                return
            }
            for _ in 0..depth {
                print!("   ");
            }
            let indicator: &str = if node.children.len()>0 && node.expand==false { "+" } else { "-" };
            let mut percentage = 0f32;
            if node.lines_instrumented != 0 {
                percentage = node.lines_covered as f32 / node.lines_instrumented as f32 * 100.0;
            }
            println!("{} {}  ({:.1}%) {}/{} ", indicator, node.name, percentage, node.lines_covered, node.lines_instrumented);
            // print children
            if node.expand {
                for c in node.children.iter() {
                    print_recursive(c, depth+1, max_depth);
                }
            }
        }
        print_recursive(self, 0, max_depth);
    }
}

fn main() {
    let matches = App::new("LCov Tool")
        .version("0.1.0")
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
    println!("The file passed is: {}", file_name);

    let print_levels = matches.value_of("print_levels").unwrap();
    let print_levels = match print_levels.parse::<i32>() {
                Ok(n) => n,
                Err(_) => -1
    };

    // read input file
    let file_content = std::fs::read_to_string(file_name).unwrap();

    // extract per-file info using a regex, and add data to the EntryTree struct
    // info on the lcov format: http://ltp.sourceforge.net/coverage/lcov/geninfo.1.php
    let re = Regex::new(r"(?x)          # ignore whitespaces and comments in regex
                          (?s)          # allow . to match \n
                          SF:(.*?)\n    # extract filename
                          (?:.*?)       # ignore function and line data
                          LH:(.*?)\n    # extract number of lines covered
                          LF:(.*?)\n    # extract number of lines instrumented
                          end_of_record").unwrap();
    let mut tree = EntryTree::new("commander");
    for cap in re.captures_iter(&file_content) {
        // println!("File: {} {} {}", &cap[1], &cap[2], &cap[3]);
        tree.get_or_create_with_path(&cap[1], cap[2].parse::<u32>().unwrap(), cap[3].parse::<u32>().unwrap());
    }
    tree.update_coverage_statistics();
    // show coverage info
    tree.print_tree(print_levels);

    // sample 
    // let mut tree2 = EntryTree::new("/");
    // tree2.get_or_create_with_path("simulation/interface/data_logging/data_logger.hpp", 10, 15);
    // tree2.get_or_create_with_path("stream.hpp", 12,12);
    // tree2.get_or_create_with_path("simulation/interface/types/simulation/named_colors.hpp", 13,14);
    // tree2.update_coverage_statistics();
    // tree2.print_tree(-1);

}
