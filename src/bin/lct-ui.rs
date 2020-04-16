use clap;
use lct::parser;

mod util;
use crate::util::event::{Event, Events};
use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListState, Paragraph, Text},
    Terminal,
};

// simple app struct to store UI state
struct App {
    tree: parser::EntryTree,
    path: Vec<String>,
}

impl App {

    fn new_from_file(file_name: &str) -> App {
        let root = parser::parse_coverage_file(file_name);
        let mut path = Vec::new();
        // add root
        path.push(root.name.clone());
        // add first level
        if root.children.len()>0 {
            let first_child = &root.children[0];
            path.push(first_child.name.clone());
        }
        App {
            tree: root,
            path: path,
        }
    }

    fn fullpath(&self, idx: i32) -> String {
        if self.path.len()>0 {
            let m = self.path.len() as i32;
            let levels = ((idx % m) + m) % m;
            let mut path_vec: Vec<String> = Vec::new();
            for i in 0..(levels+1) as usize {
                path_vec.push(self.path[i].clone());
            }
            path_vec.join("/")
        } else {
            // return the root of the tree
            String::from(self.tree.name.clone())
        }
    }

    fn get_with_index(&mut self, idx: i32) -> Option<&mut parser::EntryTree> {
        let fp = self.fullpath(idx);
        self.tree.get_with_path(&fp)
    }

    fn next(&mut self) {
        if let Some(n) = self.tree.get_with_path(&self.fullpath(-1)) {
            if n.children.len()>n.index_selected as usize {
                // only expand if the child had childrens...
                self.path.push(n.children[n.index_selected as usize].name.clone());
                if let Some(c) = self.tree.get_with_path(&self.fullpath(-1)) {
                    if c.children.is_empty() {
                        self.path.pop();
                    }
                }
            }
        }
    }

    fn previous(&mut self) {
        if self.path.len()>1 {
            self.path.pop();
        }
    }

    fn up(&mut self) {
        if let Some(n) = self.tree.get_with_path(&self.fullpath(-1)) {
            if n.children.is_empty() || n.index_selected<=1{
                n.index_selected = 0;
            } else {
                n.index_selected -=1;
            }
        }
    }

    fn down(&mut self) {
        if let Some(n) = self.tree.get_with_path(&self.fullpath(-1)) {
            if n.children.is_empty() {
                n.index_selected = 0;
            } else {
                n.index_selected =std::cmp::min(n.index_selected+1, n.children.len()-1);
            }
        }
    }
}



fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::App::new("LCov Tool - Text User Interface")
        .version("0.2.0")
        .author("Davide Bacchet <davide.bacchet@gmail.com>")
        .about("Simple LCOV coverage file parser and interactive explorer")
        .arg(clap::Arg::with_name("file")
                 .short("f")
                 .long("file")
                 .takes_value(true)
                 .help("name of the LCOV coverage file"))
        .get_matches();

    let file_name = matches.value_of("file").unwrap();
    println!("Extracting coverage information from: {}", file_name);

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    // App
    let mut app = App::new_from_file(file_name);

    loop {
        terminal.draw(|mut f| {
            // define the layout
            let vchunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Max(30)].as_ref())
                .split(f.size());

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
                .split(vchunks[1]);

            let text = [
                Text::raw(app.fullpath(-1)),
            ];
            let par = Paragraph::new(text.iter())
                .block(Block::default().title("Path").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .wrap(true);            
            f.render_widget(par, vchunks[0]);

            fn create_text_with_style(elem: &parser::EntryTree, width: usize) -> Text {
                let lines = format!("{c}/{i}", c=elem.lines_covered, i=elem.lines_instrumented);
                let perc = elem.lines_covered as f32/elem.lines_instrumented as f32*100f32;
                let s = format!("{name:width$}  {lines} {d:5.1}%", 
                    name=&elem.name, 
                    lines=lines, 
                    d=perc,
                    width=width - lines.len() - 9);
                if perc>=90.0 {
                    Text::styled(s, Style::default().fg(Color::Green))
                } else if perc>=75.0 {
                    Text::styled(s, Style::default().fg(Color::LightYellow))
                } else {
                    Text::styled(s, Style::default().fg(Color::Red))
                }
            }

            // parent folder
            if app.path.len()<2 { // have to do this check because the app will never return an empty node
                let items1 = List::new([""].iter().map(|_i| create_text_with_style(&app.tree, chunks[0].width as usize -2)))
                    .block(Block::default().borders(Borders::ALL).title("root"))
                    ;
                f.render_widget(items1, chunks[0]);
            } else  if let Some(pld) = app.get_with_index(-2) {
                let items1 = List::new(pld.children.iter().map(|i| create_text_with_style(i, chunks[0].width as usize -2)))
                    .block(Block::default().borders(Borders::ALL).title(&pld.name))
                    ;
                f.render_widget(items1, chunks[0]);
            }
            // current folder/file
            if let Some(pld) = app.get_with_index(-1) {
                let items1 = List::new(pld.children.iter().map(|i| create_text_with_style(i, chunks[1].width as usize -1)))
                    .block(Block::default().borders(Borders::TOP | Borders::BOTTOM).title(&pld.name))//"current"))
                    .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray).modifier(Modifier::BOLD))
                    .highlight_symbol(">")
                    ;
                let mut state = ListState::default();
                state.select(Some(pld.index_selected));
                f.render_stateful_widget(items1, chunks[1], &mut state);
            }
            // children of the selected item
            if let Some(pld) = app.get_with_index(-1) {
                if pld.index_selected < pld.children.len() {
                    // if the selected item has children, show them
                    let child = &pld.children[pld.index_selected];
                    let items1 = List::new(child.children.iter().map(|i| create_text_with_style(i, chunks[2].width as usize -2)))
                        .block(Block::default().borders(Borders::ALL).title(&child.name))
                        ;
                    f.render_widget(items1, chunks[2]);
                }
            }

        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Right | Key::Char('l') => {
                    app.next();
                }
                Key::Left | Key::Char('h') => {
                    app.previous();
                }
                Key::Down | Key::Char('j') => {
                    app.down();
                }
                Key::Up | Key::Char('k') => {
                    app.up();
                }
                _ => {}
            },
            Event::Tick => {
            }
        }
    }

    Ok(())
}
