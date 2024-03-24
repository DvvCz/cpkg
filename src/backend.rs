pub trait Backend {
	fn compile(&self, file: &std::path::Path, deps: &[std::path::PathBuf], to: &std::path::Path) -> std::io::Result<()>;
}

pub struct GccBackend;

impl Backend for GccBackend {
	fn compile(&self, file: &std::path::Path, deps: &[std::path::PathBuf], to: &std::path::Path) -> std::io::Result<()> {
		std::process::Command::new("gcc")
			.arg(file)
			.arg("-o")
			.arg(to)
			.output()?;

		Ok(())
	}
}

pub struct ClangBackend;

impl Backend for ClangBackend {
	fn compile(&self, file: &std::path::Path, deps: &[std::path::PathBuf], to: &std::path::Path) -> std::io::Result<()> {
		std::process::Command::new("clang")
			.arg(file)
			.arg("-o")
			.arg(to)
			.output()?;

		Ok(())
	}
}

pub fn find_backend() -> anyhow::Result<Box<dyn Backend>> {
	match which::which("gcc") {
		Ok(_) => Ok(Box::new(GccBackend)),
		Err(_) => match which::which("clang") {
			Ok(_) => Ok(Box::new(ClangBackend)),
			Err(_) => Err(anyhow::anyhow!("Couldn't find gcc or clang."))
		}
	}
}