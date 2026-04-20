use crate::constants::GURP_LIB_IMAGE;
use crate::convert;
use common::constants::{RUN_CMDS, RUN_SAFE_CMDS};
use janetrs::{Janet, JanetArgs, JanetString};
use std::process::Command;

#[janetrs::janet_fn(arity(fix(1)))]
pub fn to_json(config: &mut [Janet]) -> Janet {
    let json_string = convert::janet_to_json(&config[0]).to_string();
    Janet::wrap(json_string.as_str())
}

// Janet strings/buffers are binary-safe, so we can dump an image into one
#[janetrs::janet_fn()]
pub fn gurp_library(_arg: &mut [Janet]) -> Janet {
    let lib_as_string = JanetString::new(GURP_LIB_IMAGE);
    Janet::string(lib_as_string)
}

// run-cmd accepts a single string which describes a command, arguments and all. If the
// exact command is defined in RUN_SAFE_CMDS, it is executed, returning stdout, if there
// is any, and stderr if not. run-safe-cmd and run-cmd have to be defined as stubs in
// janet/src/dsl.janet for the library to build successfully, but they are overloaded by
// these at runtime.
//
#[janetrs::janet_fn(arity(range(1)))]
pub fn run_safe_cmd(args: &mut [Janet]) -> Janet {
    // panic is fine here as there's a guard on the number of args
    let cmd_jstr: JanetString = args.get_or_panic(0);
    let cmd_str = cmd_jstr.to_str_lossy();

    if !RUN_SAFE_CMDS.contains(&cmd_str.as_ref()) {
        janetrs::jpanic!("{} is not a permitted safe command", cmd_str);
    }

    let mut args = cmd_str.split_ascii_whitespace();

    let mut cmd = if let Some(command) = &args.next() {
        Command::new(command)
    } else {
        janetrs::jpanic!("no command given to run-safe-cmd");
    };

    while let Some(arg) = &args.next() {
        cmd.arg(arg);
    }

    exec_and_return(&mut cmd)
}

// run-cmd checks the first argument (i.e. the command to run) is in the RUN_CMDS list and,
// if it is, runs the command with the rest of the arguments. Therefore it is much more
// permissive than run-safe-cmd.
#[janetrs::janet_fn(arity(range(1)))]
pub fn run_cmd(args: &mut [Janet]) -> Janet {
    // panic is fine here as there's a guard on the number of args
    let command_arg: JanetString = args.get_or_panic(0);
    let command_arg_str = command_arg.to_str_lossy();

    if !RUN_CMDS.is_empty() && !RUN_CMDS.contains(&command_arg_str.as_ref()) {
        janetrs::jpanic!("{} is not a permitted command", command_arg_str);
    }

    let mut cmd = Command::new(command_arg_str.as_ref());

    for i in 1..args.len() {
        if let Some(arg) = args.get_value(i) {
            cmd.arg(arg.to_string());
        }
    }

    exec_and_return(&mut cmd)
}

fn exec_and_return(cmd: &mut Command) -> Janet {
    tracing::debug!(safe_command = common::cmd::to_string(cmd));

    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_owned();
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_owned();

            if !stdout.is_empty() {
                JanetString::new(stdout).into()
            } else {
                JanetString::new(stderr).into()
            }
        }
        Err(e) => janetrs::jpanic!("failed to execute: {}", e),
    }
}
