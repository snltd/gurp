# Compile-Time Sandbox

Gurp very deliberately offers no arbitrary command doer along the lines of
`ansible.builtin.command` or Puppet's `exec`.

But, configuration is written in Janet, and Janet _is_ able to run subprocesses,
unlink files, make network calls, and all the other things you expect from a
real programming language. We don't want people sneaking shell scripts into the
compile phase, or writing fork bombs to take down a server.

Flexibility always comes with a cost, and to mitigate that cost, Janet provides
[sandboxing](https://janetdocs.org/core-api/sandbox).

When runs its sandbox disallows everything except:

- `:fs-read` so files can `include` and `use` other files.
- `:fs` because `:fs-read` will not work without it.
- `:compile` so config can be evaluated.
- `:env` because it's very useful to have in client mode.

If people could set the sandboxing rules at runtime, there would be little point
having them. So, they are hard-coded into Gurp as
`SANDBOX_FORBIDDEN_CAPABILITIES` in `common/src/constants.rs`. To change them,
you must build your own binary. This is trivial once you install Rust and GCC,
so we offer no apology.

## Command Execution

The downside of sandboxing is that you lose the ability to run genuinely useful
commands which could be used in dynamic config. To help with this, Gurp offers:

- `run-safe-command` takes a single string of a command and its arguments. If
  that string exists in a hardcoded list, it is executed by the Rust backend,
  and its output returned as a Janet `String`. Gurp uses this to get facts about
  the system. For instance, we want to allow `dladm show-link`, but not
  `dladm delete-phys`. Again, the list is hardcoded in
  `common/src/constants.rs`, this time as `RUN_SAFE_CMDS`.
- `run-cmd` takes multiple arguments, the first being a command and the rest
  being its (optional) arguments. As above, but this time, only the _command_ is
  checked. For instance we might want to allow `curl` with any URL. This time
  your constant is `RUN_CMDS`, in the usual place.
- `run-any` executes any given command, but directly from the Janet interpreter:
  therefore it only works if you have allowed `:subprocess` in the sandbox.
