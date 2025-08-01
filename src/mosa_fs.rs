use std::fs;
use std::path::PathBuf;
use crate::parser::structs::ModulePathMode;

pub fn read_from_path(path: String, root: String, relative_root: String, mode: Option<ModulePathMode>) -> String {
    // dbg!(format!(
    //     "{}{}{}.mosa",
    //     root,
    //     relative_root,
    //     path.replace(":", "/")
    // ));
    if let Some(ModulePathMode::Static(st)) = mode {
        fs::read_to_string(
            format!(
                "{}{}.mosa",
                st.to_str().unwrap(),
                path.replace(":", "/")
            )
        ).unwrap()    
    } else {
        fs::read_to_string(format!(
            "{}{}{}.mosa",
            root,
            relative_root,
            path.replace(":", "/")
        ))
            .unwrap()
    }
}

pub fn relative_from(path: String, mode: Option<ModulePathMode>) -> String {
    let p = format!("{}.mosa", path.replace(":", "/"));

    let mut path_buf1 = PathBuf::from(p);
    let mut path_buf = if let Some(ModulePathMode::Static(st)) = mode {
        st
    } else {
        PathBuf::new()
    };

    path_buf1.pop();

    path_buf.push(path_buf1);

    let mut p = path_buf.to_str().unwrap().to_string();

    p.push('/');

    p
}
