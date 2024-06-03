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
			if msg.find("multiple definition of `main").is_some() {
				/* todo: should be backend agnostic, moved upward */
				anyhow::bail!("{msg}\n(cpkg: did you mean to run with --bin?)");
			} else {
				anyhow::bail!("{msg}");
			}
		}

		Ok(())
	}
}

const SUPPORTED: &[(&'static str, fn() -> Box<dyn Compiler>)] = &[
	("gcc", || Box::new(Gcc { bin: "gcc" })),
	("clang", || Box::new(Gcc { bin: "clang" })),
	("cosmocc", || Box::new(Gcc { bin: "cosmocc" })),
];

/// Tries to find an available C compiler backend.
/// Currently only supports gcc -> clang.
pub fn try_locate(proj: Option<&crate::Project>) -> anyhow::Result<Box<dyn Compiler>> {
	let default = proj
		.map(|p| {
			p.config()
				.compiler
				.as_ref()
				.map(|f| f.default.as_ref())
				.flatten()
		})
		.flatten();

	let backends = if let Some(d) = default {
		match d.as_ref() {
			"clang" | "gcc" | "cosmocc" => {
				let mut c = SUPPORTED.to_vec();
				let target = c.iter().position(|e| e.0 == d).unwrap();
				c.swap(0, target);
				std::borrow::Cow::Owned(c)
			}

			_ => {
				anyhow::bail!("Unrecognized default compiler: {d}");
			}
		}
	} else {
		std::borrow::Cow::Borrowed(SUPPORTED)
	};

	for (bin, make) in backends.as_ref() {
		if which::which(bin).is_ok() {
			return Ok(make());
		}
	}

	Err(anyhow::anyhow!("Couldn't find a compiler backend."))
}
