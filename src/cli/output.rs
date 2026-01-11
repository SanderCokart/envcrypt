//! Output control utilities for verbosity management.

/// Output control configuration
pub struct OutputConfig {
    silent: bool,
    quiet: bool,
    verbose_level: u8,
}

impl OutputConfig {
    /// Create a new output configuration from CLI flags
    pub fn new(silent: bool, quiet: bool, verbose: u8) -> Self {
        Self {
            silent,
            quiet,
            verbose_level: verbose,
        }
    }

    /// Check if errors should be displayed
    pub fn should_show_error(&self) -> bool {
        !self.silent
    }

    /// Check if info messages should be displayed
    pub fn should_show_info(&self) -> bool {
        !self.silent && !self.quiet
    }

    /// Check if verbose messages should be displayed
    pub fn should_show_verbose(&self) -> bool {
        !self.silent && !self.quiet && self.verbose_level >= 2
    }

    /// Check if debug messages should be displayed
    pub fn should_show_debug(&self) -> bool {
        !self.silent && !self.quiet && self.verbose_level >= 3
    }
}

/// Print an info message (shown unless quiet/silent)
pub fn info(config: &OutputConfig, message: &str) {
    if config.should_show_info() {
        println!("{}", message);
    }
}

/// Print a verbose message (shown at verbosity level 2+)
pub fn verbose(config: &OutputConfig, message: &str) {
    if config.should_show_verbose() {
        println!("{}", message);
    }
}

/// Print a debug message (shown at verbosity level 3+)
pub fn debug(config: &OutputConfig, message: &str) {
    if config.should_show_debug() {
        eprintln!("[DEBUG] {}", message);
    }
}
