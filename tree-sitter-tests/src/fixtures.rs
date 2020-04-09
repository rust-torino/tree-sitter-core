use crate::{
    error::Error,
    generate::generate_parser_in_directory,
    tests::helpers::dirs::*,
    util::{clone_or_pull_repository, get_treesitter_repository},
};
use once_cell::sync::OnceCell;
use rayon::prelude::*;
use std::{fs, path::PathBuf};
use walkdir::WalkDir;

static CELL: OnceCell<Result<(), Error>> = OnceCell::new();

#[derive(Debug)]
struct Grammar {
    name: &'static str,
    branch: &'static str,
}

impl Grammar {
    const fn new_master(name: &'static str) -> Self {
        Self {
            name,
            branch: "master",
        }
    }

    fn get_grammar_directories(&self) -> impl Iterator<Item = Result<PathBuf, walkdir::Error>> {
        let base_dir = GRAMMARS_DIR.join(self.name);
        WalkDir::new(base_dir)
            .into_iter()
            .filter_entry(|entry| entry.file_name() != ".git")
            .filter(|entry| match entry {
                Ok(entry) => entry.file_type().is_dir(),
                Err(_) => true,
            })
            .filter_map(|entry| match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let js_path = path.join("grammar.js");

                    if js_path.exists() {
                        Some(Ok(path.to_owned()))
                    } else {
                        None
                    }
                }
                Err(err) => Some(Err(err)),
            })
    }
}

pub fn prepare() -> Result<(), &'static Error> {
    CELL.get_or_init(|| {
        const GRAMMAR_BASE_URL: &str = "https://github.com/tree-sitter/tree-sitter-";

        fs::create_dir_all(&*GRAMMARS_DIR)?;

        const GRAMMARS: [Grammar; 14] = [
            Grammar::new_master("bash"),
            Grammar::new_master("c"),
            Grammar::new_master("cpp"),
            Grammar::new_master("embedded-template"),
            Grammar::new_master("go"),
            Grammar::new_master("html"),
            Grammar::new_master("javascript"),
            Grammar::new_master("jsdoc"),
            Grammar::new_master("json"),
            Grammar::new_master("php"),
            Grammar::new_master("python"),
            Grammar::new_master("ruby"),
            Grammar::new_master("rust"),
            Grammar::new_master("typescript"),
        ];

        for grammar in GRAMMARS.iter() {
            let directory = GRAMMARS_DIR.join(grammar.name);
            let url = format!("{}{}", GRAMMAR_BASE_URL, grammar.name);
            clone_or_pull_repository(&url, Some(grammar.branch), &directory)?;
        }

        GRAMMARS
            .par_iter()
            .flat_map(|grammar| grammar.get_grammar_directories().par_bridge())
            .map(|grammar| grammar.expect("error while listing grammar directories"))
            .try_for_each(|grammar_dir| {
                let mut grammar_path = grammar_dir.join("src");
                grammar_path.push("grammar.json");
                assert!(grammar_path.exists());

                generate_parser_in_directory(
                    &grammar_dir,
                    Some(
                        grammar_path
                            .to_str()
                            .expect("Unable to convert grammar path to UTF-8 string"),
                    ),
                    true,
                    None,
                )
            })?;

        if !TEST_GRAMMARS_DIR.exists() {
            let tree_sitter_repo_path = get_treesitter_repository()?;

            let test_grammars_dir = tree_sitter_repo_path
                .join("test")
                .join("fixtures")
                .join("test_grammars");
            assert!(test_grammars_dir.exists());

            WalkDir::new(&test_grammars_dir)
                .into_iter()
                .filter(|entry| {
                    entry
                        .as_ref()
                        .map(|entry| !entry.path().is_dir())
                        .unwrap_or(true)
                })
                .try_for_each(|entry| -> Result<(), std::io::Error> {
                    let path = entry?.into_path();
                    let relative_path = path.strip_prefix(&test_grammars_dir).unwrap();
                    let new_file = TEST_GRAMMARS_DIR.join(relative_path);

                    let new_file_dir = new_file.parent().unwrap();
                    match new_file_dir.exists() {
                        true => assert!(new_file_dir.is_dir()),
                        false => fs::create_dir_all(new_file_dir)?,
                    }

                    fs::copy(&path, &new_file)?;
                    Ok(())
                })?;
        }

        Ok(())
    })
    .as_ref()
    .map(|_| {})
}
