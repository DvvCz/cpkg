pub trait Format {
	fn format(&self, proj: &crate::Project) -> anyhow::Result<()>;
}

pub struct ClangFormat;

impl Format for ClangFormat {
	fn format(&self, proj: &crate::Project) -> anyhow::Result<()> {
		let paths = proj.src_files()
			.collect::<Vec<_>>();

		let cmd = std::process::Command::new("clang-format")
			.args(paths)
			.arg("-i") // Format in place (edit files)
			.output()?;

		if cmd.status.success() {
			Ok(())
		} else {
			Err(anyhow::anyhow!(
				"Failed to format files. {}",
				String::from_utf8_lossy(&cmd.stderr)
			))
		}
	}
}

pub struct Uncrustify;

impl Format for Uncrustify {
	fn format(&self, proj: &crate::Project) -> anyhow::Result<()> {
		let paths = proj.src_files()
			.collect::<Vec<_>>();

		let mut cmd = std::process::Command::new("uncrustify");

		if let Some(ref f) = proj.config().formatter {
			if let Some(ref u) = f.uncrustify {
				cmd
					.arg("-c")
					.arg(&u.config);
			}
		}

		let cmd = cmd
			.args(paths)
			.arg("--no-backup")
			.output()?;

		if cmd.status.success() {
			Ok(())
		} else {
			Err(anyhow::anyhow!(
				"Failed to format files. {}",
				String::from_utf8_lossy(&cmd.stderr)
			))
		}
	}
}

const SUPPORTED: &[(&'static str, fn() -> Box<dyn Format>)] = &[
	( "clang-format", || Box::new(ClangFormat) ),
	( "uncrustify", || Box::new(Uncrustify) )
];

/// Tries to find an available C formatter
/// Currently only supports clang-format.
pub fn try_locate(proj: &crate::Project) -> anyhow::Result<Box<dyn Format>> {
	let default = proj.config().formatter
		.as_ref()
		.map(|f| f.default.as_ref())
		.flatten();

	let backends = if let Some(d) = default {
		match d.as_ref() {
			"clang-format" | "uncrustify" => {
				let mut c = SUPPORTED.to_vec();
				let target = c.iter().position(|e| e.0 == d).unwrap();
				c.swap(0, target);
				std::borrow::Cow::Owned(c)
			},

			_ => {
				anyhow::bail!("Unrecognized default formatter: {d}");
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

	Err(anyhow::anyhow!("Couldn't find a formatting backend"))
}
