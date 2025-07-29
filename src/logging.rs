use crate::mosa_fs;
use colored::Colorize;
use log::log;

pub fn error_stack_traced_parser(message: String, line: usize, column: usize, file_name: String) {
    let file = mosa_fs::read_from_path(
        file_name.clone(),
        "/home/nakernarolino/RustroverProjects/mountain_sakura/input/".to_string(),
        "".to_string(),
    );

    let line_text = file.lines().collect::<Vec<&str>>()[line - 1].to_string();

    println!(
        "{}: {} {} {}: {}:{}",
        "[PARSING] [ERROR]".red(),
        message,
        "@".bright_yellow(),
        file_name.bright_green(),
        line.to_string().bright_yellow(),
        column.to_string().bright_yellow()
    );
    println!("{}", line_text);
    println!("{}{}", " ".repeat(column.max(1) - 1), "^".bright_yellow());
}

pub fn error_interpreter(message: String) {
    println!("{}: {}", "[INTERPRETING] [ERROR]".red(), message);
}

#[macro_export]
macro_rules! err {
    ($f:expr, $l:expr, $c:expr, $($arg:tt)*) => {
        crate::logging::error_stack_traced_parser(format!($($arg)*), $l, $c, $f.into())
    };

    (ft $t:expr, $($arg:tt)*) => {
        crate::logging::error_stack_traced_parser(format!($($arg)*), $t.line, $t.column, $t.file_name)
    };

    (intrp $($arg:tt)*) => {
        crate::logging::error_interpreter(format!($($arg)*))
    }
}

pub use err;
