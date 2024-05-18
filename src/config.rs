use std::collections::HashMap;

nestify::nest! {
	#[derive(serde::Serialize, serde::Deserialize)]*
	pub struct Config {
		pub package: pub struct ConfigPackage {
			pub name: String,
			/// Optional location to output the target binary
			pub bin: Option<std::path::PathBuf>
		},

		#[serde(default)]
		pub dependencies: HashMap<String, #[serde(untagged)] pub enum ConfigDependency {
			Path {
				path: std::path::PathBuf,
			},
			Git {
				git: String
			}
		}>,

		#[serde(default)]
		pub scripts: HashMap<String, String>,

		pub compiler: Option<pub struct ConfigCompiler {
			pub default: Option<String>,
			pub flags: Option<Vec<String>>,

			pub gcc: Option<pub struct ConfigGcc {
				pub flags: Option<Vec<String>>,
			}>,

			pub clang: Option<pub struct ConfigClang {
				pub flags: Option<Vec<String>>,
			}>
		}>,

		pub formatter: Option<pub struct ConfigFormatter {
			pub default: Option<String>,

			pub clang_format: Option<pub struct ConfigClangFormat {
				/* nada */
			}>,
			pub uncrustify: Option<pub struct ConfigUncrustify {
				pub config: std::path::PathBuf
			}>
		}>,

		pub docgen: Option<pub struct ConfigDocgen {
			pub default: Option<String>,

			pub doxygen: Option<pub struct ConfigDoxygen {
				pub doxyfile: std::path::PathBuf
			}>,
		}>
	}
}
