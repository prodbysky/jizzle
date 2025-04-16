pub fn display_error(
    f: &mut std::fmt::Formatter,
    line: &str,
    pos: (usize, usize),
    len: usize,
    message: &str,
) -> Result<(), std::fmt::Error> {
    writeln!(f, "Error at line {}, column {}:", pos.0, pos.1)?;
    writeln!(f, "    {line}")?;
    writeln!(
        f,
        "{:>width$}{:^<len$}",
        "^",
        "",
        width = pos.1 + 4,
        len = len.saturating_sub(0)
    )?;
    writeln!(f, "{message}")
}
