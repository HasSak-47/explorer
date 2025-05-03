use std::{
    env::current_dir, fmt::Display, fs::DirEntry, os::unix::fs::MetadataExt, path::PathBuf, time::{Duration, SystemTime, UNIX_EPOCH}
};

use anyhow::{Result, anyhow};
use clap::ValueEnum;
use mlua::{Either, FromLua, ObjectLike, Table};

use crate::fmt::{format_dir, format_file, format_link};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Color{
    BLACK = 30,
    RED = 31,
    GREEN = 32,
    YELLOW = 33,
    BLUE = 34,
    MAGENTA = 35,
    CYAN = 36,
    #[default]
    WHITE = 37,

    RGB(u8, u8, u8)
}

impl FromLua for Color {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Table(t) => {
                let r : u8 = t.get(1)?;
                let g : u8 = t.get(2)?;
                let b : u8 = t.get(3)?;

                return Ok(Color::RGB(r, g, b));
            }
            mlua::Value::String(s) => {
                return Ok(match s.to_string_lossy().as_str() {
                    "BLACK"   | "black"   => Color::BLACK,
                    "RED"     | "red"     => Color::RED,
                    "GREEN"   | "green"   => Color::GREEN,
                    "YELLOW"  | "yellow"  => Color::YELLOW,
                    "BLUE"    | "blue"    => Color::BLUE,
                    "MAGENTA" | "magenta" => Color::MAGENTA,
                    "CYAN"    | "cyan"    => Color::CYAN,
                    "WHITE"   | "white"   => Color::WHITE,
                    _ => return Err(mlua::Error::FromLuaConversionError { from: "mlua::Value", to: "Color".to_string(), message: Some("not a valid name".to_string()) })
                });
            },
            _ => {
                let v = value.to_string()?;
                return Err(mlua::Error::FromLuaConversionError {
                    from: "mlua::Value",
                    to: "Color".to_string(),
                    message: Some(format!("\"{v}\" is not valid value"))
                });
            },
        }
    }
}

#[derive(ValueEnum, Debug, Default, Clone)]
pub enum SortBy {
    #[default]
    Name,
    Type,
}

#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub enum EntryType {
    #[default]
    File,
    Dir,
    SymLink,
}

#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Hash, Clone)]
pub enum FileType {
    #[default]
    GenericFile,
    GenericDir,
    GenericSymLink,
    OtherDir(String),
    OtherFile(String),
}

#[allow(dead_code)]
pub type Permissions = u32;
#[allow(dead_code)]
const READ: Permissions = 0b001;
#[allow(dead_code)]
const WRITE: Permissions = 0b010;
#[allow(dead_code)]
const EXEC: Permissions = 0b100;

#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub ty: EntryType,

    // pub user_p  : Permissions,
    // pub group_p : Permissions,
    // pub global_p: Permissions,

    // pub user: String,
    // pub group: String,
    #[allow(dead_code)]
    pub size: u64,
    #[allow(dead_code)]
    pub date: SystemTime,

    pub childs: Vec<Entry>,
}

pub fn process_path(path: PathBuf, hidden: bool, depth: u64) -> Result<Entry>{
    let path = if path.is_relative() {
        let mut cwd = current_dir()?;
        cwd.extend(&path);

        cwd
    }else{ path };
    let name = path.file_name().unwrap().to_str().unwrap().to_string();

    // WARN: PROBABLY NOT A GOOD IDEA TO DO IT LIKE THIS
    if name.chars().next().ok_or(anyhow!("no first char?"))? == '.' && !hidden{
        return Err(anyhow!("error to ignore :)"));
    }

    let meta = path.metadata()?;

    let size = meta.size();
    let date = UNIX_EPOCH + Duration::new(meta.ctime() as u64, 0);

    let mut childs = Vec::new();
    let ty = if path.is_dir() {
        if depth > 0 {
            childs = read_dir(&path, hidden, depth - 1)?;
        }
        EntryType::Dir
    } else if path.is_symlink() {
        EntryType::SymLink
    } else {
        EntryType::File
    };

    Ok(Entry{ name, path, size, date, ty, childs })
}

fn process_entry(entry : std::io::Result<DirEntry>, hidden: bool, depth: u64) -> Result<Entry>{
    let entry = entry?;
    return process_path(entry.path(), hidden, depth);
}

pub fn read_dir(path: &PathBuf, hidden: bool, depth: u64) -> Result<Vec<Entry>>{
    let mut v = Vec::new();
    let dir = std::fs::read_dir(path)?;

    for entry in dir {
        if let Ok(k) = process_entry(entry, hidden, depth) {
            v.push(k);
        }
    }

    return Ok(v);
}

pub struct Cell {
    pub chr: char,
    pub col: Color,
}

pub struct Format {
    pub v: Vec<Cell>,
    pub childs: Vec<Format>,
}

impl From<&str> for Format {
    fn from(value: &str) -> Self {
        let mut v = Vec::new();
        for chr in value.chars() {
            v.push(Cell {
                chr,
                col: Color::WHITE,
            });
        }

        return Format {
            v,
            childs: Vec::new(),
        };
    }
}

impl FromLua for Format {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::String(s) => {
                return Ok(Format::from(s.to_string_lossy().as_str()));
            }
            mlua::Value::Table(t) => {
                let mut v = Vec::new();
                for kv in t.pairs() {
                    let (_, val): (usize, Table) = kv?;

                    let chr: String = val.get("chr")?;
                    let chr = chr.chars().into_iter().next().ok_or(anyhow!("no char?"))?;
                    let col : Color = val.get("col")?;
                    v.push(Cell { chr, col })
                }

                return Ok(Format {
                    v,
                    childs: Vec::new(),
                });

            },
            _ => {
                return Err(mlua::Error::FromLuaConversionError { from: "any", to: "Format".to_string(), message: Some("idk man".to_string()) });
            }
        }
    }
}

impl TryFrom<Entry> for Format {
    type Error = anyhow::Error;

    fn try_from(entry: Entry) -> Result<Self> {
        let mut childs = Vec::new();
        let formatter = if let EntryType::File = entry.ty {
            format_file(&entry.path)
        } else if let EntryType::Dir = entry.ty {
            for child in entry.childs {
                childs.push(Format::try_from(child)?);
            }
            format_dir(&entry.path)
        } else {
            format_link(&entry.path)
        };

        let mut fmt = formatter.call::<Format>((entry.name, entry.path, 0))?;
        fmt.childs = childs;
        return Ok(fmt);
    }
}

fn rec_format_format(
    fmt: &Format,
    depth: u64,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}{}", "\t".repeat(depth as usize), fmt)?;
    for child in &fmt.childs {
        rec_format_format(&child, depth + 1, f)?;
    }
    return Ok(());
}

impl Display for Color{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            Self::BLACK => write!(f, "\x1b[30m")?,
            Self::RED => write!(f, "\x1b[31m")?,
            Self::GREEN => write!(f, "\x1b[32m")?,
            Self::YELLOW => write!(f, "\x1b[33m")?,
            Self::BLUE => write!(f, "\x1b[34m")?,
            Self::MAGENTA => write!(f, "\x1b[35m")?,
            Self::CYAN => write!(f, "\x1b[36m")?,
            Self::WHITE => write!(f, "\x1b[37m")?,
            Self::RGB(r,g,b)=> write!(f, "\x1b[38;2;{r};{g};{b}m")?
        }

        Ok(())
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pc = Color::WHITE;
        if f.alternate() {
            return rec_format_format(self, 0, f);
        }

        for cell in &self.v {
            if cell.col != pc {
                pc = cell.col;
                write!(f, "{pc}")?;
            }
            write!(f, "{}", cell.chr)?;
        }

        if pc != Color::WHITE {
            write!(f, "\x1b[0m")?;
        }

        return Ok(());
    }
}
