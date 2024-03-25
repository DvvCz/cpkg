pub struct CompilerFlags {
	/// Whether to invoke the compiler in "repl" mode, aka silence everything that isn't an error
	repl: bool,
}

impl CompilerFlags {
	pub const REPL: Self = Self { repl: true };
}

impl Default for CompilerFlags {
	fn default() -> Self {
		Self {
			repl: false
		}
	}
}

pub trait Compiler {
	fn compile(&self, file: &std::path::Path, deps: &[std::path::PathBuf], to: &std::path::Path, flags: &CompilerFlags) -> anyhow::Result<()>;
}

pub struct Gcc {
	path: std::path::PathBuf
}

impl Compiler for Gcc {
	fn compile(&self, file: &std::path::Path, _deps: &[std::path::PathBuf], to: &std::path::Path, flags: &CompilerFlags) -> anyhow::Result<()> {
		let mut cmd = std::process::Command::new(&self.path);

		let mut cmd = cmd
			.arg(file)
			.arg("-o")
			.arg(to);

		if flags.repl {
			cmd = cmd.arg("-w");
		}

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
			Ok(path) => Ok(Box::new(Gcc { path })), // api should be compatible enough to my knowledge.
			Err(_) => Err(anyhow::anyhow!("Couldn't find gcc or clang."))
		}
	}
}