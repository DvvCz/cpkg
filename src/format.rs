pub trait Format {
	fn format(&self, src: &std::path::Path) -> anyhow::Result<()>;
}

pub struct ClangFormat;

impl Format for ClangFormat {
	fn format(&self, src: &std::path::Path) -> anyhow::Result<()> {
		let paths = walkdir::WalkDir::new(src)
			.into_iter()
			.flat_map(std::convert::identity) // Filter out walkdir failures
			.filter(|e| e.path().extension().filter(|ext| *ext == "c" || *ext == "h").is_some()) // Only formatting .c and .h files
			.map(|e| e.path().to_owned()); // Retrieving paths of files

		let cmd = std::process::Command::new("clang-format")
			.args(paths)
			.arg("-i") // Format in place (edit files)
			.output()?;

		if cmd.status.success() {
			Ok(())
		} else {
			Err(anyhow::anyhow!("Failed to format files. {}", String::from_utf8_lossy(&cmd.stderr)))
		}
	}
}

/// Tries to find an available C formatter
/// Currently only supports clang-format.
pub fn try_locate() -> anyhow::Result<Box<dyn Format>> {
	match which::which("clang-format") {
		Ok(_) => Ok(Box::new(ClangFormat)),
		Err(_) => Err(anyhow::anyhow!("Couldn't find clang-format."))
	}
}