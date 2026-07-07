use anyhow::Context;
use camino::Utf8Path;
use common::types::{ApplyOutputOpts, ApplyVmOpts, CompileOpts};
use embed::compiler::{self, ConfigCompiler};
use std::fs;
use std::process::ExitCode;

pub fn run(host_file: &Utf8Path, opts: &CompileOpts) -> ExitCode {
    let compiler = match compiler::ConfigCompiler::new(
        &ApplyVmOpts::default(),
        false,
        ApplyOutputOpts::default(),
        Some(host_file),
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("cannot create ConfigCompiler: {e:#}");
            return ExitCode::FAILURE;
        }
    };

    let result = match opts.format.as_str() {
        "json" => compile_to_json(&compiler, host_file, opts),
        "janet" => compile_to_janet(&compiler, host_file, opts),
        "jimage" => compile_to_image(host_file, opts),
        _ => {
            tracing::error!("format must be janet, json or jimage");
            return ExitCode::FAILURE;
        }
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("error compiling JSON: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn compile_to_json(
    compiler: &ConfigCompiler,
    path: &Utf8Path,
    opts: &CompileOpts,
) -> anyhow::Result<()> {
    let compiled = compiler.janet_file(path, true)?;

    if let Some(out_file) = &opts.output_file {
        fs::write(out_file, compiled)
            .with_context(|| format!("error writing JSON to {out_file}"))?;
        tracing::info!("wrote JSON to {out_file}");
    } else {
        println!("{compiled}");
    }

    Ok(())
}

fn compile_to_janet(
    compiler: &ConfigCompiler,
    path: &Utf8Path,
    opts: &CompileOpts,
) -> anyhow::Result<()> {
    let compiled = compiler.janet_file(path, false)?;

    if let Some(out_file) = &opts.output_file {
        fs::write(out_file, compiled)
            .with_context(|| format!("error writing Janet to {out_file}"))?;
        tracing::info!("wrote Janet to {out_file}");
    } else {
        println!("{compiled}");
    }

    Ok(())
}

fn compile_to_image(path: &Utf8Path, opts: &CompileOpts) -> anyhow::Result<()> {
    let output_path = &opts
        .output_file
        .as_ref()
        .context("writing an image requires an output path")?;

    let image_data = compiler::to_jimage(path).context("error compiling image file")?;

    fs::write(output_path, image_data)
        .with_context(|| format!("error writing image file {output_path}"))?;

    tracing::info!("wrote image file to '{output_path}'");
    Ok(())
}
