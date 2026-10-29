use std::process::Command;

/// Creates a new Command with Windows-specific configuration to hide console windows.
/// On Windows, this prevents the "terminal opening" issue that users report during
/// FFmpeg operations (normalization, video creation, etc.).
pub fn create_hidden_command<S: AsRef<std::ffi::OsStr>>(program: S) -> Command {
    #[allow(unused_mut)] // mut needed for Windows-specific creation_flags() call
    let mut cmd = Command::new(program);
    
    // On Windows, set CREATE_NO_WINDOW flag to prevent console window from appearing
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hidden_command() {
        // Test that the function creates a command without panicking
        let cmd = create_hidden_command("echo");
        assert_eq!(cmd.get_program(), "echo");
    }
}