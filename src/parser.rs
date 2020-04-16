use regex::Regex;

#[derive(Debug)]
pub struct EntryTree {
    name: String,
    lines_covered: u32,
    lines_instrumented: u32,
    expand: bool,
    children: Vec<EntryTree>,
}

impl EntryTree {
    // create a tree
    pub fn new(root_name: &str) -> EntryTree {
        EntryTree {
            name : String::from(root_name),
            lines_covered: 0,
            lines_instrumented: 0,
            expand : true,
            children : Vec::new(),
        }
    }

    // get or create an entry 
    pub fn get_or_create(&mut self, name: &str) -> &mut EntryTree {
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

    pub fn get_or_create_with_path(&mut self, path: &str, lines_covered: u32, lines_instrumented: u32) -> &mut EntryTree {
        let tokens = path.split("/");
        let mut node = self;
        for t in tokens {
            node = node.get_or_create(t);
        }
        node.lines_covered = lines_covered;
        node.lines_instrumented = lines_instrumented;
        node
    }

    pub fn update_coverage_statistics(&mut self) {
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

    pub fn print_tree(&self, max_depth: i32) {
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


/// parse a coverage file and generate a tree with coverage data per folder/file
pub fn parse_coverage_file(file_name: &str) -> EntryTree {
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
    tree
}
