// You specify packages by name, so `ooce/editor/helix` rather than
// `pkg://sysdef/ooce/editor/helix@25.1-151052.0:20250108t110907Z`. This means you
// can't request specific versions. I might change this, but I never pin to
// version, and I'm immediately only solving the problems I actually have.

// Operating only on name makes the doer run faster, because it knows exactly
// what can and cannot be done, so runs `pkg(5)` in the most efficient way
// possible `pkg(5)` is rather a slow tool.

use std::process::Command;

fn installed_packages() -> anyhow::Result<String> {
    let cmd = Command::new("/bin/pkg")
        .arg("list")
        .arg("-aH")
        .arg("-o")
        .arg("name,flags")
        .output()?;

    Ok(String::from_utf8(cmd.stdout)?)
}

type PackageName = String;

// TODO this needs a better name
struct GlobalPackages {
    available: Vec<PackageName>,
    installed: Vec<PackageName>,
}

fn parse_pkg_output(output: &str) -> GlobalPackages {
    let mut installed: Vec<String> = Vec::new();
    let mut available: Vec<String> = Vec::new();

    for l in output.trim().lines() {
        let bits: Vec<_> = l.split_whitespace().collect();

        if bits.len() != 2 {
            continue;
        }

        if bits[1].starts_with('i') {
            installed.push(bits[0].to_owned());
        } else {
            available.push(bits[0].to_owned());
        }
    }

    GlobalPackages {
        available,
        installed,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::load_fixture;

    #[test]
    fn test_parse_pkg_output() {
        let result = parse_pkg_output(&load_fixture("doers/package/pkg-output"));
        assert_eq!(613, result.installed.len());
        assert_eq!(521, result.available.len());
    }
    // #[test]
    // fn test_packages_to_add() {
    //     ["helix" "janet" "oozone"]
    //     ["helix" "rust" "zcage"]
    //     ["helix" "janet" "vim" "flac" "lame"])
    //   @["janet"])
    //   }
}
