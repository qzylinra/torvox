// @Shell environment setup, IMPL_TERM_005, impl, [REQ_TERM_005]
// @need-ids: REQ_TERM_005
pub struct ShellEnv {
    pub home: String,
    pub user: String,
    pub path: String,
    pub working_directory: String,
    pub prefix: Option<String>,
    pub extra: Vec<(String, String)>,
}

impl Default for ShellEnv {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_else(|_| "root".to_string());
        let path = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string());
        let working_directory = home.clone();
        Self {
            home,
            user,
            path,
            working_directory,
            prefix: None,
            extra: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_env_default_reads_from_std_env() {
        let env = ShellEnv::default();
        assert!(!env.home.is_empty());
        assert!(!env.user.is_empty());
        assert!(!env.path.is_empty());
        assert!(!env.working_directory.is_empty());
        assert!(env.extra.is_empty());
    }
}
