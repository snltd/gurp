[![Test Rust](https://github.com/snltd/gurp/actions/workflows/test-rust.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-rust.yml)
[![Janet Tests](https://github.com/snltd/gurp/actions/workflows/test-janet.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-janet.yml)

# gurp

## What?

Gurp is an almost certainly doomed attempt to write an illumos configuration
management tool driven by Lisp.

## Why?

I run illumos, and I want some kind of config management. Chef is too heavy,
Puppet doesn't really support illumos any more, CfEngine is too much work, and
Ansible is Ansible.

## How?

A user writes machine configurations using a thin
[Janet](https://janet-lang.org/) DSL. Resources such as Unix users, SMF
services, or ZFS datasets have their properties described as Janet tables, but
can, of course, be wrapped in, or contain, arbitrary Janet code. Resources may
reference properties of other resources.

## When?

Maybe sometime, maybe not. It's very much an experimental side-project.

## Who?

Gurp. Because why not?

## NOTES

Currently needs `CFLAGS="-D__EXTENSIONS__ -std=c99" cargo build` to compile.
