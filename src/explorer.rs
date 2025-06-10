use std::{cmp::min, env::current_dir, io, path::PathBuf, thread, time::Duration};

use anyhow::Result;
use crossterm::{
    execute,
    event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{backend::CrosstermBackend, buffer::Cell, layout::{Position, Rect}, style, widgets::{List, ListState}, Terminal};

use crate::util::{Format, read_dir, Color};

fn set_color(cell: &mut Cell, color: &Color) {
    cell.set_fg(match color{
        Color::RED => style::Color::Red,
        Color::GREEN => style::Color::Green,
        Color::YELLOW => style::Color::Yellow,
        Color::BLUE => style::Color::Blue,
        Color::MAGENTA => style::Color::Magenta,
        Color::CYAN => style::Color::Cyan,
        Color::RGB(r,g,b) => style::Color::Rgb(*r, *g, *b),
        _ => style::Color::White,
    });
}

impl ratatui::widgets::Widget for &Format {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        for i in 0..min(area.width, self.v.len() as u16) {
            let val = &self.v[i as usize];
            if let Some(cell) = buf.cell_mut(Position::new( area.x + i, area.y )){
                cell.set_char(val.chr);
                set_color(cell, &self.v[i as usize].col);
            }
        }
    }
}

#[derive(Default)]
pub struct Explorer {
    cache: Vec<Format>,
    cwd: PathBuf,
    state: ListState,
}


impl Explorer {
    pub fn new() -> Self {
        let cwd = current_dir().unwrap();
        let cache = read_dir(&cwd, false, 0)
            .unwrap()
            .into_iter()
            .map(|k| Format::try_from(k).unwrap())
            .collect();

        Explorer { cwd, cache, ..Default::default()}
    }

    pub fn render(self) -> Result<()> {
        render(self)
    }

    pub fn update(&mut self) {}

    pub fn move_up(&mut self) {
        if let Some(s) = self.state.selected_mut(){
            *s += 1;
        }
        else{
            self.state.select(Some(0));
        }
    }

    pub fn move_down(&mut self) {
        if let Some(s) = self.state.selected_mut(){
            if *s > 0 {
                *s -= 1;
            }
        }
        else{
            self.state.select(Some(0));
        }
    }
}

fn render(mut ex: Explorer) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    'render: loop {
        terminal.draw(|f| {
            let size = f.area();
            let items = ex.cache.iter();
            let list = List::new(items);
            f.render_widget(&ex, size);
        })?;
        let e = event::read()?;
        use crossterm::event::Event as cE;
        use crossterm::event as ce;
        match e{
            cE::Key(k) => {
                match k.code{
                    ce::KeyCode::Esc => {
                        break 'render;
                    },
                    ce::KeyCode::Char(c) => {
                        match c{
                            'k' => { ex.move_up(); },
                            'j' => { ex.move_down(); },
                            _ => {},
                        }
                    }
                    _ => {},
                }
            }
            _ => {},
        }

        thread::sleep(Duration::from_millis(100/6));
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
