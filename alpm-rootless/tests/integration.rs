//! Integration tests for [`alpm_rootless`].

mod utils {
    use alpm_rootless::get_command;
    use testresult::TestResult;

    /// Ensures that the "whoami" command can be found on a Linux system.
    #[test]
    #[cfg(target_os = "linux")]
    fn get_command_succeeds() -> TestResult {
        let command = "whoami";
        if let Err(error) = get_command(command) {
            panic!("Should have found command \"{command}\", but got error instead:\n{error}")
        };

        Ok(())
    }

    /// Ensures that a command unlikely to ever exist cannot be found on a Linux system.
    #[test]
    #[cfg(target_os = "linux")]
    fn get_command_fails() -> TestResult {
        let command = "d202d7951df2c4b711ca44b4bcc9d7b363fa4252127e058c1a910ec05b6cd038d71cc21221c031c0359f993e746b07f5965cf8c5c3746a58337ad9ab65278e77";

        if let Ok(path) = get_command(command) {
            panic!("Should not have found command {path:?}, but succeeded");
        };

        Ok(())
    }
}
