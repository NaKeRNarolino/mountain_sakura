use std::fs;
use std::path::PathBuf;

pub fn read_from_path(path: String, root: String, relative_root: String) -> String {
    // dbg!(format!(
    //     "{}{}{}.mosa",
    //     root,
    //     relative_root,
    //     path.replace(":", "/")
    // ));
    fs::read_to_string(format!(
        "{}{}{}.mosa",
        root,
        relative_root,
        path.replace(":", "/")
    ))
    .unwrap()
}

pub fn relative_from(path: String) -> String {
    let p = format!("{}.mosa", path.replace(":", "/"));

    let mut path_buf = PathBuf::from(p);

    path_buf.pop();

    let mut p = path_buf.to_str().unwrap().to_string();

    p.push('/');

    p
}
