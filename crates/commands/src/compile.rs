use anyhow::Context;
use camino::Utf8Path;
use common::info;
use common::types::{ApplyOutputOpts, ApplyVmOpts, CompileOpts};
use embed::compiler::{self, ConfigCompiler};
use std::fs;
use std::process::ExitCode;

pub fn run(host_file: &Utf8Path, opts: &CompileOpts) -> ExitCode {
    tracing::debug!("creating ConfigCompiler");

    let compiler = match compiler::ConfigCompiler::new(
        &ApplyVmOpts::default(),
        false,
        ApplyOutputOpts {
            colour: opts.colour,
            line_no: opts.line_no,
            ..Default::default()
        },
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("cannot create ConfigCompiler: {e:#}");
            return ExitCode::FAILURE;
        }
    };

    tracing::debug!("compiling source");

    let result = match opts.format.as_str() {
        "json" => compile_to_json(&compiler, host_file, opts),
        "janet" => compile_to_janet(&compiler, host_file, opts),
        "jimage" => compile_to_image(&compiler, host_file, opts),
        _ => {
            tracing::error!("format must be janet, json or jimage");
            return ExitCode::FAILURE;
        }
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("error compiling to {}: {e:#}", opts.format);
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
        write_file(out_file, &compiled, "JSON")?
    } else {
        print_output(&compiled, opts.line_no)
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
        write_file(out_file, &compiled, "janet")?
    } else {
        print_output(&compiled, opts.line_no)
    }

    Ok(())
}

fn compile_to_image(
    compiler: &ConfigCompiler,
    path: &Utf8Path,
    opts: &CompileOpts,
) -> anyhow::Result<()> {
    let output_path = &opts
        .output_file
        .as_ref()
        .context("writing an image requires an output path")?;

    let image_data =
        compiler::to_jimage(Some(&compiler.client), path).context("error compiling image file")?;

    fs::write(output_path, image_data)
        .with_context(|| format!("error writing image file {output_path}"))?;

    tracing::info!("wrote image file to '{output_path}'");
    Ok(())
}

fn write_file(path: &Utf8Path, compiled: &str, fmt: &str) -> anyhow::Result<()> {
    fs::write(path, compiled).with_context(|| format!("error writing {fmt} to {path}"))?;
    tracing::info!("wrote {fmt} to {path}");

    Ok(())
}

fn print_output(output: &str, line_nos: bool) {
    if line_nos {
        println!(
            "{}",
            info::dump_config(
                output,
                None,
                &ApplyOutputOpts {
                    line_no: true,
                    ..Default::default()
                },
            )
        )
    } else {
        println!("{output}");
    }
}
