use std::{cmp::Ordering, env::current_dir, path::PathBuf};

use crate::{fmt::{format_dir, format_file}, util::*};

use anyhow::{anyhow, Result};
use clap::Parser;
use mlua::{Either, Table};

#[derive(Debug, Parser, Clone)]
pub struct List{
    #[arg(value_enum, long, short, default_value_t=SortBy::Name)]
    sort_by: SortBy,

    #[arg(long, short, default_value_t=0)]
    recursive: u64,

    #[arg(long, short, default_value_t=true)]
    list: bool,

    #[arg(long, default_value_t=false)]
    hidden: bool,
}

fn sort_name(a: &Entry, b: &Entry) -> std::cmp::Ordering{
    let v = a.name.cmp(&b.name);
    if let Ordering::Equal = v {
        return a.ty.cmp(&b.ty);
    }
    return v;
}

fn sort_type(a: &Entry, b: &Entry) -> std::cmp::Ordering{
    let v = a.ty.cmp(&b.ty);
    if let Ordering::Equal = v {
        return a.name.cmp(&b.name);
    }
    return v;
}

impl List{
    pub fn ls(&self) -> Result<()>{
        let list = self.list == true || self.recursive > 1;

        let cwd = current_dir()?;
        let mut entries = read_dir(&cwd, self.hidden, self.recursive)?;
        match &self.sort_by{
            SortBy::Name => entries.sort_by(sort_name),
            SortBy::Type => entries.sort_by(sort_type),
        }

        let mut v = Vec::new();
        for entry in entries{
            v.push(Format::try_from(entry)?);
        }

        if list{
            for e in v{
                print!("{e:#}");
            }
        }

        return Ok(());
    }
}
