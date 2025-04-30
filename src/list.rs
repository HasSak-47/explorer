use std::{cmp::Ordering, path::PathBuf, process::exit };

use crate::{get_options, util::*};

use anyhow::Result;
use clap::Parser;

#[allow(dead_code)]
fn curr_dir() -> Vec<PathBuf> {
    use std::env::current_dir;
    return match current_dir(){
        Ok(s) => vec![s],
        Err(_) => {
            exit(-1);
        },
    };
}

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

    #[arg(default_values_os_t = curr_dir())]
    paths: Vec<PathBuf>,
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

    fn get_formats(&self, path: &PathBuf) -> Result<Vec<Format>>{
        let mut entries = read_dir(&path, self.hidden, self.recursive)?;
        match &self.sort_by{
            SortBy::Name => entries.sort_by(sort_name),
            SortBy::Type => entries.sort_by(sort_type),
        }

        let mut v = Vec::new();
        for entry in entries{
            v.push(Format::try_from(entry)?);
        }

        return Ok(v);
    }

    pub fn ls(&self) -> Result<()>{
        let list = self.list == true || self.recursive > 0 || self.paths.len() > 0;
        if get_options().debug {
            println!("rec: {}", self.recursive);
            println!("list: {}", self.list);
            println!("flist: {}", list);
            
        }

        let mut v = Vec::new();
        if self.paths.len() > 1 {
            for path in &self.paths{
                match process_path(path.clone(), self.hidden, 0) {
                    Ok(k) => {
                        let root = Format::try_from(process_path(k.path.clone(), self.hidden, self.recursive + 1)?)?;
                        v.push(root);
                    },
                    _ => {},
                }
            }
        }
        else{
            v = self.get_formats(&self.paths[0])?;
        }

        if list{
            for e in v{
                print!("{e:#}");
            }
        }
        else{
            let (cols, _rows) = crossterm::terminal::size()?;
            let mut current = 0;
            let max = v.iter().map(|f| f.v.len()).max().unwrap() + 1;
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
