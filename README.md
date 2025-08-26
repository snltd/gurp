[![Rust Tests](https://github.com/snltd/gurp/actions/workflows/test-rust.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-rust.yml)
[![Janet Tests](https://github.com/snltd/gurp/actions/workflows/test-janet.yml/badge.svg)](https://github.com/snltd/gurp/actions/workflows/test-janet.yml)

# gurp

## What?

Gurp is an illumos configuration management tool.

## Why?

I run illumos, and I want config management. Chef is too heavy; Puppet doesn't
really support illumos any more, even with Oracle's providers; CfEngine is too
much work; and Ansible is Ansible.

## How?

A user defines machine configurations in a thin [Janet](https://janet-lang.org/)
DSL. Resources such as Unix users, SMF services, or ZFS datasets are described
as Janet structs, but can, of course, be wrapped in, or contain, arbitrary code.
Resources may reference properties of other resources.

## Who?

Gurp. Because why not?

## NOTES

Currently needs `CFLAGS="-D__EXTENSIONS__ -std=c99" cargo build` to compile.
