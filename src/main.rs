use ccometixline::cli::Cli;
use ccometixline::config::{Config, ConfigLoader, InputData};
use ccometixline::core::StatusLineGenerator;
use std::io;

#[cfg(windows)]
fn is_stdin_piped() -> bool {
    use winapi::um::fileapi::GetFileType;
    use winapi::um::winbase::{FILE_TYPE_PIPE, FILE_TYPE_DISK, STD_INPUT_HANDLE};
    use winapi::um::processenv::GetStdHandle;
    
    unsafe {
        let handle = GetStdHandle(STD_INPUT_HANDLE);
        let file_type = GetFileType(handle);
        // 文件重定向时类型是 FILE_TYPE_DISK，管道时是 FILE_TYPE_PIPE
        file_type == FILE_TYPE_PIPE || file_type == FILE_TYPE_DISK
    }
}

#[cfg(not(windows))]
fn is_stdin_piped() -> bool {
    use std::os::unix::io::AsRawFd;
    let fd = io::stdin().as_raw_fd();
    unsafe {
        let mut stat = std::mem::zeroed();
        libc::fstat(fd, &mut stat);
        (stat.st_mode & libc::S_IFIFO) != 0
    }
}

fn main() -> io::Result<()> {
    let cli = Cli::parse_args();

    // Handle special CLI modes
    if cli.print_config {
        let config = Config::default();
        println!("{}", toml::to_string(&config).unwrap());
        return Ok(());
    }

    if cli.validate {
        println!("Configuration validation not implemented yet");
        return Ok(());
    }

    if cli.configure {
        println!("TUI configuration mode not implemented yet");
        return Ok(());
    }
    
    // Load configuration
    let config = ConfigLoader::load();

    // Check if stdin is piped
    if !is_stdin_piped() {
        // No piped input
        println!("请通过管道提供输入数据");
        return Ok(());
    }

    // Read Claude Code data from stdin
    let stdin = io::stdin();
    let input: InputData = serde_json::from_reader(stdin.lock())?;

    // Generate statusline
    let generator = StatusLineGenerator::new(config);
    let statusline = generator.generate(&input);
    println!("{}", statusline);

    Ok(())
}
