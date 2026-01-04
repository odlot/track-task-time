use std::io::{self, Write};

pub fn prompt_yes_no(message: &str) -> bool {
    print!("{}", message);
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

pub fn prompt_line(message: &str) -> Result<String, String> {
    print!("{}", message);
    io::stdout().flush().map_err(|err| err.to_string())?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| err.to_string())?;
    Ok(input.trim().to_string())
}

pub fn prompt_optional(message: &str) -> Result<Option<String>, String> {
    let input = prompt_line(message)?;
    if input.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}
