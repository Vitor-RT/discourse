pub struct Middleware;

impl Middleware {
    pub fn policy_preventing_framing(policy: &str) -> String {
        let mut directives: Vec<&str> = policy.split(';').map(|s| s.trim()).collect();
        directives.retain(|directive| !directive.starts_with("frame-ancestors "));
        directives.push("frame-ancestors 'none'");
        directives.join("; ")
    }
}
