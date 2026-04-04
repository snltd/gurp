use anyhow::Context;
use camino::Utf8PathBuf;
use common::types::{ApplyOpts, CompileOpts};
use embed::compiler;
use std::fs;
use std::process::ExitCode;

pub fn run(host_file: &Utf8PathBuf, c_opts: &CompileOpts, opts: &ApplyOpts) -> ExitCode {
    match c_opts.format.as_str() {
        "json" => match compile_json(host_file, c_opts, opts) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                tracing::error!("error compiling JSON: {e:#}");
                ExitCode::FAILURE
            }
        },
        "janet" => match compile_janet(host_file, c_opts, opts) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                tracing::error!("error compiling JSON: {e:#}");
                ExitCode::FAILURE
            }
        },
        "jimage" => match compile_jimage(host_file, c_opts, opts) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                tracing::error!("error compiling JSON: {e:#}");
                ExitCode::FAILURE
            }
        },
        _ => {
            tracing::error!("format must be janet, json or jimage");
            ExitCode::FAILURE
        }
    }
}

fn compile_json(
    host_file: &Utf8PathBuf,
    c_opts: &CompileOpts,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let json = compiler::local_janet_to_json(host_file, opts)?;

    if let Some(out_file) = &c_opts.output_file {
        fs::write(out_file, json).with_context(|| format!("error writing JSON to {out_file}"))?;
        tracing::info!("wrote JSON to {out_file}");
    } else {
        println!("{json}");
    }

    Ok(())
}

fn compile_janet(
    host_file: &Utf8PathBuf,
    c_opts: &CompileOpts,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let json = compiler::local_janet_to_janet(host_file, opts)?;

    if let Some(out_file) = &c_opts.output_file {
        fs::write(out_file, json).with_context(|| format!("error writing JSON to {out_file}"))?;
        tracing::info!("wrote JSON to {out_file}");
    } else {
        println!("{json}");
    }

    Ok(())
}

fn compile_jimage(
    host_file: &Utf8PathBuf,
    c_opts: &CompileOpts,
    opts: &ApplyOpts,
) -> anyhow::Result<()> {
    let output_path = &c_opts
        .output_file
        .as_ref()
        .context("writing an image requires an output path")?;

    let image_data =
        compiler::local_janet_to_jimage(host_file, opts).context("error compiling image file")?;

    fs::write(output_path, image_data)
        .with_context(|| format!("error writing image file {output_path}"))?;

    tracing::info!("wrote image file to '{output_path}'");
    Ok(())
}
