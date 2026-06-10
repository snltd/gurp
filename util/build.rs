use vergen_git2::{Build, Cargo, Emitter, Git2};

fn main() {
    let build = Build::all_build();
    let cargo = Cargo::all_cargo();
    let git = Git2::all_git();

    Emitter::default()
        .add_instructions(&build)
        .unwrap()
        .add_instructions(&cargo)
        .unwrap()
        .add_instructions(&git)
        .unwrap()
        .emit()
        .unwrap();
}
