use std::io::Write;

pub fn write_text(lines: &[String], w: &mut impl Write) -> anyhow::Result<()> {
    for line in lines {
        writeln!(w, "{}", line)?;
    }
    Ok(())
}
