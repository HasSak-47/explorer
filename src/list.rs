use crate::util::*;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct List{
    #[arg(value_enum, long, short, default_value_t=SortBy::Name)]
    sort_by: SortBy,
}

impl List{
    pub fn ls(&self) -> Result<()>{
        let mut entries = read_dir()?;
        match &self.sort_by{
            SortBy::Name => entries.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::Type => entries.sort_by(|a, b| a.name.cmp(&b.name)),
        }
        Ok(())
    }
}
