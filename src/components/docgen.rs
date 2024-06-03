pub trait Docgen {
	fn generate(&self, src: &std::path::Path, to: &std::path::Path) -> anyhow::Result<()>;
	fn open(&self, to: &std::path::Path) -> anyhow::Result<()>;
}

pub struct Doxygen;

impl Docgen for Doxygen {
	fn generate(&self, _src: &std::path::Path, to: &std::path::Path) -> anyhow::Result<()> {
		let config = to.join("Doxyfile");

		#[rustfmt::skip]
		std::fs::write(
			&config,
			indoc::formatdoc! {"
				INPUT=../../src
				OUTPUT_DIRECTORY=.
			"}
		)?;

		let out = std::process::Command::new("doxygen")
			.current_dir(to)
			.output()?;

		if !out.status.success() {
			anyhow::bail!(
				"Failed to generate documentation: {}",
				String::from_utf8_lossy(&out.stderr)
			);
		}

		Ok(())
	}

	fn open(&self, to: &std::path::Path) -> anyhow::Result<()> {
		let index = to.join("html/index.html");
		start_program(&index)
	}
}

pub struct Cldoc;

impl Docgen for Cldoc {
	fn generate(&self, src: &std::path::Path, to: &std::path::Path) -> anyhow::Result<()> {
		let out = std::process::Command::new("cldoc")
			.arg("generate")
			.arg("--")
			.arg("--output")
			.arg(to)
			.arg(src)
			.output()?;

		if !out.status.success() {
			anyhow::bail!(
				"Failed to generate documentation: {}",
				String::from_utf8_lossy(&out.stderr)
			);
		}

		Ok(())
	}

	fn open(&self, _to: &std::path::Path) -> anyhow::Result<()> {
		anyhow::bail!("Open is not yet implemented for cldoc. Sorry!")
	}
}

#[cfg(target_os = "linux")]
fn start_program(p: &std::path::Path) -> anyhow::Result<()> {
	std::process::Command::new("xdg-open").arg(p).output()?;

	Ok(())
}

#[cfg(target_os = "windows")]
fn start_program(p: &std::path::Path) -> anyhow::Result<()> {
	// TODO: Test on windows
	std::process::Command::new("cmd")
		.arg("-C")
		.arg("start")
		.arg(p)
		.spawn()?
		.wait()?;

	Ok(())
}

const SUPPORTED: &[(&'static str, fn() -> Box<dyn Docgen>)] = &[
	("doxygen", || Box::new(Doxygen)),
	("cldoc", || Box::new(Cldoc)),
];

/// Tries to find an available C compiler backend.
/// Currently only supports gcc -> clang.
pub fn try_locate(proj: &crate::Project) -> anyhow::Result<Box<dyn Docgen>> {
	let default = proj.config()
		.docgen
		.as_ref()
		.map(|f| f.default.as_ref())
		.flatten();

	let backends = if let Some(d) = default {
		match d.as_ref() {
			"doxygen" | "cldoc" => {
				let mut c = SUPPORTED.to_vec();
				let target = c.iter().position(|e| e.0 == d).unwrap();
				c.swap(0, target);
				std::borrow::Cow::Owned(c)
			},

			_ => {
				anyhow::bail!("Unrecognized default doc generator: {d}");
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

	Err(anyhow::anyhow!("Couldn't find a docgen backend"))
}
