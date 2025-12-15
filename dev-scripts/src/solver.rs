use std::{
    collections::HashSet,
    io,
    io::{Read, Write},
};

use alpm_common::MetadataFile;
use alpm_db::desc::DbDescFile;
use alpm_repo_db::desc::RepoDescFile;
use alpm_solve::{ALPMDependencyProvider, DependencyResolutionAction, Solution, Solver};
use alpm_types::{Name, RepositoryName};
use log::debug;

use crate::{cache::CacheDir, error::Error};

fn ask(question: &str) -> bool {
    print!("{} [y/n]:", question);
    io::stdout().flush().expect("TODO");
    loop {
        let mut input = [0];
        let _ = std::io::stdin().read(&mut input);
        match input[0] as char {
            'y' | 'Y' => return true,
            'n' | 'N' => return false,
            _ => {
                print!("\nPlease answer 'y' or 'n'.: ");
                io::stdout().flush().expect("TODO");
            }
        }
    }
}

pub fn solve_upgrade(
    cache_dir: CacheDir,
    partial: bool,
    strict_optional: bool,
) -> Result<(), Error> {
    // .conf
    let repositories = vec![("core", 0), ("extra", -1), ("multilib", -2)];

    println!(":: Parsing system state...");
    let mut system_state = Vec::new();
    for installed_pkg in std::fs::read_dir("/var/lib/pacman/local/").unwrap() {
        let pkg_path = installed_pkg.unwrap().path();
        let desc_path = pkg_path.join("desc");
        match DbDescFile::from_file(&desc_path) {
            Ok(desc) => system_state.push(desc),
            Err(e) => debug!("oops: {:?}\n{}", desc_path, e),
        };
    }

    let mut provider =
        ALPMDependencyProvider::new(&system_state).with_optdepends_enforced(strict_optional);

    println!(":: Parsing sync dbs...");
    for repo in repositories {
        let mut sync_db = Vec::new();
        let repo_path = cache_dir.as_ref().join("databases").join(repo.0);
        for package_entry in std::fs::read_dir(repo_path).unwrap() {
            let pkg_path = package_entry.unwrap().path();
            let desc_path = pkg_path.join("desc");
            match RepoDescFile::from_file(&desc_path) {
                Ok(desc) => sync_db.push(desc),
                Err(e) => debug!("oops: {:?}\n{}", desc_path, e),
            }
        }
        provider.add_package_repository(RepositoryName::new(repo.0)?, repo.1, sync_db);
    }

    provider.add_installed(system_state.clone());
    let mut solver: Solver = provider.into();
    let mut dep_locks: HashSet<Name> = HashSet::new();
    let mut partial = partial;

    loop {
        println!(":: Solving dependencies...");
        let solution = solver.upgrade(system_state.clone(), dep_locks.clone(), !partial);

        let mut must_resolve = false;

        let solution: Result<Solution, Error> = match solution {
            Ok(solution) => {
                for action in solution.as_ref() {
                    if let DependencyResolutionAction::Remove { name, .. } = action {
                        println!(":: Action required.\n {action}");
                        let proceed = ask(format!("Remove {name}?").as_str());
                        if !proceed {
                            must_resolve = true;
                            dep_locks.insert(name.clone());
                            break;
                        }
                    }
                }
                Ok(solution)
            }
            Err(e) => {
                if !partial {
                    partial = ask("Can't resolve dependencies. Attempt partial upgrade?");
                    if partial {
                        must_resolve = true;
                    }
                }
                Err(e.into())
            }
        };

        if !must_resolve {
            println!(":: System upgrade:\n{}", solution?);
            break;
        }
    }

    Ok(())
}
