pub trait Compiler {
	fn compile(
		&self,
		files: &[std::path::PathBuf],
		deps: &[&std::path::Path],
		to: &std::path::Path,
		flags: &[String],
	) -> anyhow::Result<()>;

	fn makefile(&self, proj: &crate::Project) -> String;
}

pub struct Gcc {
	bin: &'static str,
}

impl Compiler for Gcc {
	fn makefile(&self, proj: &crate::Project) -> String {
		let cc = self.bin;

		let name = proj.name();
		let flags = proj.build_flags(self as &dyn Compiler).join(" ");
		let bin = proj.build_out(None).display().to_string();

		indoc::formatdoc! {"
			CC = {cc}

			{name}: $(wildcard src/*)
				$(CC) $(wildcard src/*.c) -o {bin} {flags}
		"}
	}

	fn compile(
		&self,
		files: &[std::path::PathBuf],
		deps: &[&std::path::Path],
		to: &std::path::Path,
		flags: &[String],
	) -> anyhow::Result<()> {
		let mut cmd = std::process::Command::new(&self.bin);

		cmd.args(files).arg("-o").arg(to).args(flags);

		for dep in deps {
			// Include dependency folder
			cmd.arg("-I").arg(dep);
		}

		let e = cmd.output()?;

		if !e.status.success() {
			let msg = String::from_utf8_lossy(&e.stderr);
			if msg.find("multiple definition of `main").is_some() { /* todo: should be backend agnostic, moved upward */
				anyhow::bail!("{msg}\n(cpkg: did you mean to run with --bin?)");
			} else {
				anyhow::bail!("{msg}");
			}
		}

		Ok(())
	}
}

/// Tries to find an available C compiler backend.
/// Currently only supports gcc -> clang.
pub fn try_locate() -> anyhow::Result<Box<dyn Compiler>> {
	match which::which("gcc") {
		Ok(_) => Ok(Box::new(Gcc { bin: "gcc" })),
		Err(_) => match which::which("clang") {
			Ok(_) => Ok(Box::new(Gcc { bin: "clang" })), // Should be api compatible.
			Err(_) => Err(anyhow::anyhow!("Couldn't find gcc or clang.")),
		},
	}
}
