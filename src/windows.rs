pub fn key_to_string(key: &rdev::Key) -> String {
    format!("{key:?}")
        .replace("Key", "")
        .replace("Left", " L")
        .replace("Right", " R")
}
