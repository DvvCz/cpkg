pub trait Backend {
	fn compile(&self, file: &std::path::Path, deps: &[std::path::PathBuf], to: &std::path::Path) -> anyhow::Result<()>;
}

pub struct GccBackend {
	path: std::path::PathBuf
}

impl Backend for GccBackend {
	fn compile(&self, file: &std::path::Path, _deps: &[std::path::PathBuf], to: &std::path::Path) -> anyhow::Result<()> {
		let e = std::process::Command::new(&self.path)
			.arg(file)
			.arg("-o")
			.arg(to)
			.spawn()?
			.wait()?;

		if e.success() {
			Ok(())
		} else {
			Err(anyhow::anyhow!("Failed to compile file."))
		}
	}
}

/// Tries to find an available C compiler backend.
/// Currently only supports gcc -> clang.
pub fn find_backend() -> anyhow::Result<Box<dyn Backend>> {
	match which::which("gcc") {
		Ok(path) => Ok(Box::new(GccBackend { path })),
		Err(_) => match which::which("clang") {
			Ok(path) => Ok(Box::new(GccBackend { path })), // api should be compatible enough to my knowledge.
			Err(_) => Err(anyhow::anyhow!("Couldn't find gcc or clang."))
		}
	}
}