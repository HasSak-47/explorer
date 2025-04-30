use std::{cmp::min, env::current_dir, io, path::PathBuf, time::Duration, thread};

use anyhow::Result;
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use tui::{backend::CrosstermBackend, layout::Rect, style::Color, Terminal};

use crate::util::{read_dir, Format};

pub struct Explorer{
    cwd: PathBuf,
    cache: Vec<Format>,
}

impl tui::widgets::Widget for &Format{
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        for i in 0..min(area.width , self.v.len() as u16) {
            let cell = buf.get_mut(i, 0);
            let val = &self.v[i as usize];
            cell.set_char(val.chr);
            cell.set_fg(Color::Rgb(val.col.0, val.col.1, val.col.2));
        }
    }
}

impl tui::widgets::Widget for &Explorer{
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        for i in 0..min(area.height , self.cache.len() as u16) {
            let val = &self.cache[i as usize];
            val.render(Rect::new(area.x, area.y + 1, area.width, 1), buf);
        }
    }
}


impl Explorer{
    pub fn new() -> Self{
        let cwd = current_dir().unwrap();
        let cache = read_dir(&cwd, false, 0).unwrap()
            .into_iter().map(|k| Format::try_from(k).unwrap())
            .collect();

        Explorer{ cwd , cache }
    }

    pub fn render(self) -> Result<()>{
        render(self)
    }

}

fn render(ex: Explorer) -> Result<()>{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        f.render_widget(&ex, size);
    })?;

    thread::sleep(Duration::from_millis(5000));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}
