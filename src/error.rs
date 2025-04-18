pub fn display_error(
    f: &mut std::fmt::Formatter,
    input_name: Option<&str>,
    pos: (usize, usize),
    message: &str,
) -> Result<(), std::fmt::Error> {
    if let Some(n) = input_name {
        writeln!(f, "./{n}:{}:{} \n{message}", pos.0, pos.1)
    } else {
        writeln!(f, "./<input>:{}:{} \n{message}", pos.0, pos.1)
    }
}
