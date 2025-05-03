use std::{
    io::Read,
    process::{Command, Stdio},
};

use crate::{LUA, MAP, util::FileType};
use mlua::{Function, Lua, Table};

pub fn get_formats(_: &Lua, tb: mlua::Table) -> mlua::Result<()> {
    let mut map = MAP
        .lock()
        .map_err(|err| mlua::Error::RuntimeError(err.to_string()))?;
    let file_formats: Table = tb.get("file")?;

    match file_formats.get(1) {
        Ok(default) => {
            *map.get_mut(&FileType::GenericFile).unwrap() = default;
        }
        Err(_) => {}
    }
    for kv in file_formats.pairs() {
        let (k, v): (String, Function) = kv?;
        map.insert(FileType::OtherFile(k), v);
    }

    let dirs_format: Table = tb.get("dirs")?;
    match dirs_format.get(1) {
        Ok(default) => {
            *map.get_mut(&FileType::GenericDir).unwrap() = default;
        }
        Err(_) => {}
    }
    for kv in file_formats.pairs() {
        let (k, v): (String, Function) = kv?;
        map.insert(FileType::OtherDir(k), v);
    }

    Ok(())
}

pub fn bash(l: &Lua, s: String) -> mlua::Result<Table> {
    let mut cmd = Command::new("bash");
    cmd.arg("-c")
        .arg(s)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    let mut result = cmd.spawn()?;
    let stdout = match &mut result.stdout {
        Some(s) => {
            let mut buf = String::new();
            s.read_to_string(&mut buf)?;
            buf
        }
        _ => String::new(),
    };
    let stderr = match &mut result.stderr {
        Some(s) => {
            let mut buf = String::new();
            s.read_to_string(&mut buf)?;
            buf
        }
        _ => String::new(),
    };

    let r = result.wait()?.code().unwrap_or(0);
    let k = l.create_table()?;
    k.set(1, stdout)?;
    k.set(2, stderr)?;
    k.set(3, r)?;

    return Ok(k);
}
