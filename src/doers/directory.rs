use janetrs::JanetType::{String, Table};
use janetrs::{Janet, JanetArgs, TaggedJanet};

#[janetrs::janet_fn(arity(fix(2)))]
pub fn directory_is(args: &mut [Janet]) -> Janet {
    let dir_name = args.get_matches(0, &[String]).unwrap();
    let dir_opts = args.get_matches(1, &[Table]).unwrap();

    println!("opts are #{:?}", dir_opts);

    match dir_name {
        TaggedJanet::String(name) => {
            let dir_name_rs = name
                .to_str()
                .expect("Could not convert directory name to str");

            let result = check_state(dir_name_rs);
            Janet::from(result)
        }
        _ => {
            eprintln!("Directory name must be a string");
            Janet::from(false)
        }
    }
}

fn check_state(name: &str) -> bool {
    println!("checking state of {}", name);
    true
}
