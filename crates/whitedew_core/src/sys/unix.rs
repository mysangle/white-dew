
pub fn write_stdout(text: &str) {

}

pub fn apperr_format(f: &mut std::fmt::Formatter<'_>, code: u32) -> std::fmt::Result {
    write!(f, "Error {code}")?;
    
    Ok(())
}
