pub const TEMPORARY_UPLOAD_PREFIX: &str = "temp/";
pub const CACHE_MAXIMUM_SIZE: usize = 500;
pub const CACHE_EVICT_COUNT: usize = 100;

pub fn get_depth_for(id: i64) -> usize {
    if id > 0 {
        let depth = (id as f64 / 1_000.0).log(16.0).ceil() as isize;
        if depth > 0 {
            return depth as usize;
        }
    }
    0
}

pub fn get_path_for(file_type: &str, id: i64, sha: &str, extension: &str) -> String {
    let depth = get_depth_for(id);
    let mut tree = String::new();
    if depth > 0 {
        for c in sha.chars().take(depth) {
            tree.push(c);
            tree.push('/');
        }
    }
    format!("{}/{}X/{}{}{}", file_type, depth + 1, tree, sha, extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_store_depth_and_path() {
        assert_eq!(get_depth_for(0), 0);
        assert_eq!(get_depth_for(1000), 0);
        assert_eq!(get_depth_for(1001), 1);
        let path = get_path_for("uploads", 100, "abcdef1234567890", ".png");
        assert!(path.contains("uploads/"));
    }
}
