//! A database's schema, its upgrade written (open problem 41; ADR-0015,
//! amendment 2026-10-20), for every surface: the scripts that bring each
//! database on a server from the schema version it holds to a later one,
//! and the same written into a folder an operator names, never over a file.
//! The schema, its versions and the scripts are
//! `persist::storage::schema`'s; `xmip database` and
//! `Get-XmipDatabaseScript` render them, and neither knows a table.

use std::path::{Path, PathBuf};

use persist::storage::database::{Server, Technology};
use persist::storage::schema;

/// The scripts that upgrade a server's databases, in the order they run,
/// and the schema version they reach.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Upgrade {
    pub version: u32,
    /// Each script's file name and its text.
    pub scripts: Vec<(String, String)>,
}

/// The scripts for the database server `server` names — `postgresql`,
/// `mssql` or `mariadb` — from schema version `from` to `to`, zero for this
/// build's ([`schema::VERSION`]).
///
/// # Errors
///
/// A word that is no database server Xmip Storage keeps a database on, or
/// an upgrade the schema has none of: the refusal in words.
pub fn upgrade(server: &str, from: u32, to: u32) -> Result<Upgrade, String> {
    let to = if to == 0 { schema::VERSION } else { to };
    let scripts = named(server).and_then(|server| schema::upgrade(server, from, to))?;

    Ok(Upgrade {
        version: schema::VERSION,
        scripts,
    })
}

/// The database server `word` names, or the refusal naming the words.
fn named(word: &str) -> Result<Server, String> {
    match Technology::named(word) {
        Some(Technology::Server(server)) => Ok(server),
        _ => {
            let words: Vec<&str> = Server::ALL.iter().map(|server| server.word()).collect();
            Err(format!(
                "'{word}' is no database server Xmip Storage keeps a database on; write {}",
                words.join(", ")
            ))
        }
    }
}

/// `upgrade`'s scripts written into `folder`, made where it is not, each
/// as its name: the paths written.
///
/// # Errors
///
/// A script's file is there already, and nothing is written; or the folder
/// or a file could not be written.
pub fn write(upgrade: &Upgrade, folder: &Path) -> Result<Vec<PathBuf>, String> {
    let paths: Vec<PathBuf> = upgrade
        .scripts
        .iter()
        .map(|(name, _)| folder.join(name))
        .collect();

    if let Some(there) = paths.iter().find(|path| path.exists()) {
        return Err(format!(
            "REFUSED: {} is there already, and nothing is overwritten.",
            there.display()
        ));
    }

    std::fs::create_dir_all(folder)
        .map_err(|failure| format!("{}: {failure}", folder.display()))?;
    for ((_, text), path) in upgrade.scripts.iter().zip(&paths) {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .and_then(|mut file| std::io::Write::write_all(&mut file, text.as_bytes()))
            .map_err(|failure| format!("{}: {failure}", path.display()))?;
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_upgrade_that_is_none_and_a_word_that_is_no_server_are_refused_in_words() {
        let none = upgrade("postgresql", 1, 0).expect_err("this build is the first release");
        assert!(
            none.contains(&format!("1 to {} is none", schema::VERSION)),
            "{none}"
        );

        let refused = upgrade("mongodb", 1, 2).expect_err("no such server");
        assert!(
            refused.contains("write postgresql, mssql, mariadb"),
            "{refused}"
        );
    }

    #[test]
    fn scripts_are_written_into_a_folder_and_never_over_a_file() {
        let folder =
            std::env::temp_dir().join(format!("xmip-operate-database-{}", std::process::id()));
        let upgrade = Upgrade {
            version: 2,
            scripts: vec![("1-to-2.sql".to_string(), "select 1;".to_string())],
        };

        let written = write(&upgrade, &folder).expect("written");
        assert_eq!(
            std::fs::read_to_string(&written[0]).expect("read"),
            "select 1;"
        );
        let refused = write(&upgrade, &folder).expect_err("there already");
        assert!(refused.contains("nothing is overwritten"), "{refused}");
        let _ = std::fs::remove_dir_all(&folder);
    }
}
