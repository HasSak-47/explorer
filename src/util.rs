use std::{
    env::current_dir, fmt::Display, fs::DirEntry, os::unix::fs::MetadataExt, path::PathBuf, time::{Duration, SystemTime, UNIX_EPOCH}
};

use anyhow::{anyhow, Result};
use clap::ValueEnum;
use mlua::{Either, Table};

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
}

fn process_entry(entry : std::io::Result<DirEntry>) -> Result<Entry>{
    let entry = entry?;
    let path = entry.path();
    let name = path.file_name().unwrap().to_str().unwrap().to_string();
    let meta = path.metadata()?;

    let size = meta.size();
    let date = UNIX_EPOCH + Duration::new(meta.ctime() as u64, 0) ;

    let ty = if path.is_dir() { EntryType::Dir }
    else if path.is_symlink(){ EntryType::SymLink }
    else{ EntryType::File } ;

    Ok(Entry{ name, path, size, date, ty })
}

pub fn read_dir() -> Result<Vec<Entry>>{
    let mut v = Vec::new();
    let path = current_dir()?;
    let dir = std::fs::read_dir(path)?;

    for entry in dir{
        if let Ok(k) = process_entry(entry){
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
}

impl From<&str> for Format {
    fn from(value: &str) -> Self {
        let mut v = Vec::new();
        for chr in value.chars(){
            v.push(Cell{chr, col: (0xff, 0xff, 0xff, )});
        }

        return Format{v};
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
            let col: Table = val.get("col")?;

            let r : u8 = col.get(1).unwrap_or(0xff);
            let g : u8 = col.get(2).unwrap_or(0xff);
            let b : u8 = col.get(3).unwrap_or(0xff);
            let col = (r, g, b);
            v.push(Cell{ chr, col, })
        }

        return Ok(Format{v});
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

impl Display for Format{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pc = (0xff, 0xff, 0xff);

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
