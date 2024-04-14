pub trait Compiler {
	fn compile(&self, main: &std::path::Path, deps: &[&std::path::Path], to: &std::path::Path, flags: &[String]) -> anyhow::Result<()>;
	fn get_compile_command(&self, main: &std::path::Path, deps: &[&std::path::Path], to: &std::path::Path, flags: &[String]) -> std::process::Command;
}

pub struct Gcc {
	path: std::path::PathBuf
}

impl Compiler for Gcc {
	fn get_compile_command(&self, main: &std::path::Path, deps: &[&std::path::Path], to: &std::path::Path, flags: &[String]) -> std::process::Command {
		let mut cmd = std::process::Command::new(&self.path);

		cmd
			.arg(main)
			.arg("-o")
			.arg(to)
			.args(flags);

		for dep in deps { // Include dependency folder
			cmd
				.arg("-I")
				.arg(dep);
		}

		cmd
	}

	fn compile(&self, file: &std::path::Path, deps: &[&std::path::Path], to: &std::path::Path, flags: &[String]) -> anyhow::Result<()> {
		let mut cmd = self.get_compile_command(file, deps, to, flags);

		let e = cmd
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
			Ok(path) => Ok(Box::new(Gcc { path })), // Should be api compatible.
			Err(_) => Err(anyhow::anyhow!("Couldn't find gcc or clang."))
		}
	}
}