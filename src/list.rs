use std::cmp::Ordering;

use crate::{fmt::{format_dir, format_file}, util::*};

use anyhow::Result;
use clap::Parser;
use mlua::{Either, Table};

#[derive(Debug, Parser)]
pub struct List{
    #[arg(value_enum, long, short, default_value_t=SortBy::Name)]
    sort_by: SortBy,
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
        let mut entries = read_dir()?;
        match &self.sort_by{
            SortBy::Name => entries.sort_by(sort_name),
            SortBy::Type => entries.sort_by(sort_type),
        }

        let mut v = Vec::new();
        for entry in entries{
            let formatter = if let EntryType::File = entry.ty {
                format_file(&entry.path)
            }
            else {
                format_dir(&entry.path)
            };
            v.push( Format::try_from( formatter.call::<Either<Table, String>>((entry.name, entry.path, 0))?)? );
        }

        for e in v{
            println!("{}", e);
        }
        Ok(())
    }
}
