
use std::env;
use std::process;

/// Parses command-line arguments and returns the specified configuration options.
///
/// # Arguments
/// * None, uses `env::args` internally.
///
/// # Returns
/// * A tuple containing (command, option) if valid arguments are provided.
/// * Exits the process with an error code if arguments are invalid.
pub fn parse_cli_args() -> (String, Option<String>) {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: omnitensor-node <command> [option]");
        process::exit(1);
    }

    let command = args[1].clone();
    let option = if args.len() > 2 { Some(args[2].clone()) } else { None };

    (command, option)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_cli_args() {
        env::set_var("RUST_BACKTRACE", "1");
        env::set_var("TEST_ARGV_0", "omnitensor-node");
        env::set_var("TEST_ARGV_1", "start");
        let (cmd, opt) = parse_cli_args();
        assert_eq!(cmd, "start");
        assert!(opt.is_none());
    }
}
