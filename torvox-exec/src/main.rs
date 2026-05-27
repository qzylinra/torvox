fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("torvox-exec: usage: torvox-exec <command> [args...]");
        std::process::exit(1);
    }
    let cmd = &args[1];
    let cmd_args = &args[2..];
    let err = nix::unistd::execvp(
        std::ffi::CString::new(cmd.as_str()).unwrap().as_c_str(),
        &cmd_args
            .iter()
            .map(|a| std::ffi::CString::new(a.as_str()).unwrap())
            .collect::<Vec<_>>(),
    )
    .unwrap_err();
    eprintln!("exec failed: {err}");
    std::process::exit(1);
}
