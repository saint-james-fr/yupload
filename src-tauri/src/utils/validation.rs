/// Validate YouTube video title
///
/// YouTube API limits:
/// - Title cannot be empty
/// - Title cannot exceed 100 characters
///
/// Returns Ok(()) if valid, Err with message if invalid
pub fn validate_youtube_title(title: &str) -> Result<(), String> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err("Title cannot be empty".to_string());
    }
    if title.len() > 100 {
        return Err("Title cannot exceed 100 characters".to_string());
    }
    Ok(())
}

/// Validate YouTube video description
///
/// YouTube API limits:
/// - Description cannot exceed 5000 characters
///
/// Returns Ok(()) if valid, Err with message if invalid
pub fn validate_youtube_description(description: &str) -> Result<(), String> {
    if description.len() > 5000 {
        return Err("Description cannot exceed 5000 characters".to_string());
    }
    Ok(())
}

/// Validate YouTube video tags
///
/// YouTube API limits (snippet.tags):
/// - Each tag cannot exceed 30 characters
/// - Total character count across all tags cannot exceed 500
///
/// Returns Ok(()) if valid, Err with message if invalid
pub fn validate_youtube_tags(tags: &[String]) -> Result<(), String> {
    const MAX_TAG_LEN: usize = 30;
    const MAX_TOTAL_LEN: usize = 500;

    let mut total_len: usize = 0;
    for (i, tag) in tags.iter().enumerate() {
        let len = tag.len();
        if len > MAX_TAG_LEN {
            let preview = if tag.len() > 40 {
                format!("{}...", &tag[..37])
            } else {
                tag.clone()
            };
            return Err(format!(
                "Tag {} exceeds {} characters (\"{}\" has {})",
                i + 1,
                MAX_TAG_LEN,
                preview,
                len
            ));
        }
        total_len += len;
    }
    if total_len > MAX_TOTAL_LEN {
        return Err(format!(
            "Total tags length ({} characters) exceeds YouTube limit of {}",
            total_len, MAX_TOTAL_LEN
        ));
    }
    Ok(())
}

/// Validate search query
///
/// Prevents abuse and ensures reasonable query length:
/// - Query cannot be empty (after trimming)
/// - Query cannot exceed 200 characters
///
/// Returns Ok(()) if valid, Err with message if invalid
pub fn validate_search_query(query: &str) -> Result<(), String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err("Search query cannot be empty".to_string());
    }
    if query.len() > 200 {
        return Err("Search query cannot exceed 200 characters".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_youtube_title() {
        // Valid title
        assert!(validate_youtube_title("My Video Title").is_ok());

        // Empty title
        assert!(validate_youtube_title("").is_err());
        assert!(validate_youtube_title("   ").is_err());

        // Too long title
        let long_title = "a".repeat(101);
        assert!(validate_youtube_title(&long_title).is_err());

        // Exactly 100 characters (should be valid)
        let exact_title = "a".repeat(100);
        assert!(validate_youtube_title(&exact_title).is_ok());
    }

    #[test]
    fn test_validate_youtube_description() {
        // Valid description
        assert!(validate_youtube_description("My description").is_ok());
        assert!(validate_youtube_description("").is_ok()); // Empty is allowed

        // Too long description
        let long_desc = "a".repeat(5001);
        assert!(validate_youtube_description(&long_desc).is_err());

        // Exactly 5000 characters (should be valid)
        let exact_desc = "a".repeat(5000);
        assert!(validate_youtube_description(&exact_desc).is_ok());
    }

    #[test]
    fn test_validate_search_query() {
        // Valid query
        assert!(validate_search_query("test query").is_ok());

        // Empty query
        assert!(validate_search_query("").is_err());
        assert!(validate_search_query("   ").is_err());

        // Too long query
        let long_query = "a".repeat(201);
        assert!(validate_search_query(&long_query).is_err());

        // Exactly 200 characters (should be valid)
        let exact_query = "a".repeat(200);
        assert!(validate_search_query(&exact_query).is_ok());
    }

    #[test]
    fn test_validate_youtube_tags() {
        // Valid: empty
        assert!(validate_youtube_tags(&[]).is_ok());
        // Valid: single tag ≤30
        assert!(validate_youtube_tags(&["music".to_string()]).is_ok());
        assert!(validate_youtube_tags(&["a".repeat(30)]).is_ok());
        // Invalid: single tag >30
        assert!(validate_youtube_tags(&["a".repeat(31)]).is_err());
        // Valid: total ≤500, each ≤30 (16 * 30 = 480)
        let many: Vec<String> = (0..16).map(|_| "a".repeat(30)).collect();
        assert!(validate_youtube_tags(&many).is_ok());
        // Invalid: total >500 (17 * 30 = 510)
        let too_many: Vec<String> = (0..17).map(|_| "a".repeat(30)).collect();
        assert!(validate_youtube_tags(&too_many).is_err());
    }
}
