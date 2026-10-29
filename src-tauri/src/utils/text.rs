use regex::Regex;

/// Clean names by removing Discogs disambiguation numbers like "(5)"
pub fn clean_name(name: &str) -> String {
    // Remove trailing pattern like " (5)" or "(5)" at the end
    let re = Regex::new(r"\s*\(\d+\)\s*$").unwrap();
    re.replace(name, "").to_string()
}
