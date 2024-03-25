pub trait Compiler {
	fn compile(&self, file: &std::path::Path, deps: &[std::path::PathBuf], to: &std::path::Path) -> anyhow::Result<()>;
}

pub struct Gcc {
	path: std::path::PathBuf
}

impl Compiler for Gcc {
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
pub fn try_locate() -> anyhow::Result<Box<dyn Compiler>> {
	match which::which("gcc") {
		Ok(path) => Ok(Box::new(Gcc { path })),
		Err(_) => match which::which("clang") {
			Ok(path) => Ok(Box::new(Gcc { path })), // api should be compatible enough to my knowledge.
			Err(_) => Err(anyhow::anyhow!("Couldn't find gcc or clang."))
		}
	}
}