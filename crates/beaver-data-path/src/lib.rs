use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot resolve home directory")
        .join(".local/share/cl-go-dash")
}

#[cfg(test)]
mod tests {
    #[test]
    fn data_directory_keeps_the_existing_cross_platform_suffix() {
        assert!(super::data_dir().ends_with(".local/share/cl-go-dash"));
    }
}
