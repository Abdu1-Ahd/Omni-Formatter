#[derive(Debug)]
enum FormatError {
    Io(std::io::Error),
    Parse(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Io(e) => write!(f, "IO Error: {}", e),
            FormatError::Parse(e) => write!(f, "Parse Error: {}", e),
        }
    }
}
