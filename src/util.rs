use std::{
    fmt::Display, fs::DirEntry, os::unix::fs::MetadataExt, path::PathBuf, time::{Duration, SystemTime, UNIX_EPOCH}
};

use anyhow::{anyhow, Result};
use clap::ValueEnum;
use mlua::{Either, Table};

use crate::fmt::{format_dir, format_file, format_link};

#[derive(ValueEnum, Debug, Default, Clone)]
pub enum SortBy{
    #[default]
    Name,
    Type,
}

#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub enum EntryType{
    #[default]
    File,
    Dir,
    SymLink
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
const READ : Permissions = 0b001;
#[allow(dead_code)]
const WRITE: Permissions = 0b010;
#[allow(dead_code)]
const EXEC : Permissions = 0b100;

#[derive(Debug)]
pub struct Entry{
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

fn process_entry(entry : std::io::Result<DirEntry>, hidden: bool, depth: u64) -> Result<Entry>{
    let entry = entry?;
    let path = entry.path();
    let name = path.file_name().unwrap().to_str().unwrap().to_string();

    // WARN: NOT A GOOD
    if name.chars().next().ok_or(anyhow!("no first char?"))? == '.' && !hidden{
        return Err(anyhow!("error to ignore :)"));
    }

    let meta = path.metadata()?;


    let size = meta.size();
    let date = UNIX_EPOCH + Duration::new(meta.ctime() as u64, 0) ;

    let mut childs = Vec::new();
    let ty = if path.is_dir() {
        if depth > 0 {
            childs = read_dir(&path, hidden, depth - 1)?;
        }
        EntryType::Dir
    }
    else if path.is_symlink(){
        EntryType::SymLink
    }
    else{
        EntryType::File
    } ;



    Ok(Entry{ name, path, size, date, ty, childs })
}

pub fn read_dir(path: &PathBuf, hidden: bool, depth: u64) -> Result<Vec<Entry>>{
    let mut v = Vec::new();
    let dir = std::fs::read_dir(path)?;

    for entry in dir{
        if let Ok(k) = process_entry(entry, hidden, depth){
            v.push(k);
        }
    }

    return Ok(v);
}

pub struct Cell{
    pub chr : char,
    pub col : (u8, u8, u8,),
}

pub struct Format{
    pub v: Vec<Cell>,
    pub childs: Vec<Format>,
}

impl From<&str> for Format {
    fn from(value: &str) -> Self {
        let mut v = Vec::new();
        for chr in value.chars(){
            v.push(Cell{chr, col: (0xff, 0xff, 0xff, )});
        }

        return Format{v, childs: Vec::new()};
    }
}

impl TryFrom<Table> for Format {
    type Error = anyhow::Error;

    fn try_from(value: Table) -> Result<Self, Self::Error>{
        let mut v = Vec::new();
        for kv in value.pairs(){
            let (_, val): (usize, Table) = kv?;

            let chr: String = val.get("chr")?;
            let chr = chr.chars().into_iter().next().ok_or(anyhow!("no char?"))?;
            let col = match val.get::<Table>("col"){
                Ok(k) => {(
                    k.get(1).unwrap_or(0xff),
                    k.get(2).unwrap_or(0xff),
                    k.get(3).unwrap_or(0xff),
                )},
                _ => (0xff, 0xff, 0xff, ),
            };

            v.push(Cell{ chr, col, })
        }

        return Ok(Format{v, childs: Vec::new()});
    }
}

impl TryFrom<Either<Table, String>> for Format{
    type Error = anyhow::Error;
    fn try_from(value: Either<Table, String>) -> Result<Self> {
        match value {
            Either::Left(l) => Format::try_from(l),
            Either::Right(r) => Ok(Format::from(r.as_str())),
        }
    }
}

impl TryFrom<Entry> for Format{
    type Error = anyhow::Error;

    fn try_from(entry: Entry) -> Result<Self> {
        let mut childs = Vec::new();
        let formatter = if let EntryType::File = entry.ty {
            format_file(&entry.path)
        }
        else
        if let EntryType::Dir = entry.ty {
            for child in entry.childs {
                childs.push(Format::try_from(child)?);
            }
            format_dir(&entry.path)

        }
        else {
            format_link(&entry.path)
        };

        let mut fmt =  Format::try_from( formatter.call::<Either<Table, String>>((entry.name, entry.path, 0))?)?;
        fmt.childs = childs;
        return Ok(fmt);

    }
}

fn rec_format_format(fmt: &Format, depth: u64, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
    writeln!(f, "{}{}", "\t".repeat(depth as usize), fmt)?;
    for child in &fmt.childs {
        rec_format_format(&child, depth + 1, f)?;
    }
    return Ok(());
}

impl Display for Format{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pc = (0xff, 0xff, 0xff);
        if f.alternate(){
            return rec_format_format(self, 0, f);
        }

        for cell in &self.v {
            if cell.col != pc {
                pc = cell.col;
                let (r, g, b) = pc;
                write!(f, "\x1b[38;2;{r};{g};{b}m")?;
                
            }
            write!(f, "{}", cell.chr)?;
        }

        if pc != (0xff, 0xff, 0xff) {
            write!(f, "\x1b[0m")?;
        }

        return Ok(());
    }
}
