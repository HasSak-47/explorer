use crate::*;

fn _format_file<'a>(map: &'a HashMap<FileType, Function>, path: &PathBuf) -> Option<&'a Function>{
    let extension = path.extension()?.to_str()?.to_string();
    let ft = FileType::OtherFile(extension.clone());

    return map.get(&ft);
}

pub fn format_file(path: &PathBuf) -> Function{
    let map = MAP.lock().unwrap();
    let format = _format_file(&map, path)
        .unwrap_or(map.get(&FileType::GenericFile).unwrap());

    return format.clone();
}

pub fn format_dir(_: &PathBuf) -> Function{
    let map = MAP.lock().unwrap();
    let format = map.get(&FileType::GenericDir).unwrap();

    return format.clone();
}

pub fn format_link(_: &PathBuf) -> Function{
    let map = MAP.lock().unwrap();
    let format = map.get(&FileType::GenericSymLink).unwrap();

    return format.clone();
}
