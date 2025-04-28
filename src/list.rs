use std::cmp::Ordering;

use crate::{get_options, util::*};

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct List{
    #[arg(value_enum, long, short, default_value_t=SortBy::Name)]
    sort_by: SortBy,

    #[arg(long, short, default_value_t=0)]
    recursive: u64,

    #[arg(long, short, default_value_t=false)]
    list: bool,

    #[arg(long, default_value_t=false)]
    hidden: bool,
}

fn sort_name(a: &Entry, b: &Entry) -> std::cmp::Ordering{
    let v = a.name.to_lowercase().cmp(&b.name.to_lowercase());
    if let Ordering::Equal = v {
        return a.ty.cmp(&b.ty);
    }
    return v;
}

fn sort_type(a: &Entry, b: &Entry) -> std::cmp::Ordering{
    let v = a.ty.cmp(&b.ty);
    if let Ordering::Equal = v {
        return a.name.to_lowercase().cmp(&b.name.to_lowercase());
    }
    return v;
}

impl List{
    pub fn ls(&self) -> Result<()>{
        let list = self.list == true || self.recursive > 1;
        if get_options().debug {
            println!("rec: {}", self.recursive);
            println!("list: {}", self.list);
            println!("flist: {}", list);
            
        }

        let path = &get_options().path;
        let mut entries = read_dir(path, self.hidden, self.recursive)?;
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
        else{
            let (cols, _rows) = crossterm::terminal::size()?;
            let max = v.iter().map(|f| f.v.len()).max().unwrap() + 1;
            if get_options().debug {
                println!("max: {max}");
            }

            let mut current = 0;
            let cap = cols as usize / max;
            for e in v{
                if current >= cap{
                    current = 0;
                    println!();
                }
                let dif = max - e.v.len();
                print!("{}{}", e.to_string(), " ".repeat(dif));
                current += 1;
            }
            println!();

        }

        return Ok(());
    }
}
