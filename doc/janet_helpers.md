## Janet Helpers

### Functions/Macros

As well as
[all of Janet's standard library](https://janet-lang.org/api/index.html), `gurp`
gives you a few convenience functions and macros.

- `(this-host)` returns the name of the host as defined in the top level
  `(host)` declaration, **not** the result of `(uname -n)`.
- `(this-host-k)` is the output of `(this-host)`, but as a Janet keyword.
- `(hostname)` returns the hostname of the box on which your Janet is compiled,
  via `uname -n`.
- `(this-role)` returns the name of the enclosing role.
- `(this-role-k)` as above, but a keyword.
- `(this resource-type resource-name resource-property)` is a convenient way to
  return a reference to a resource in the current role. For example,
  `(this "user" "rob" "uid")`.
- `(template-out template var-struct)` takes a template string and a struct or
  table, and replaces references in the template with their corresponding values
  in the struct. For instance,
  `(template "{{ prog }} is {{ thing }}" {:prog "Janet" :thing "nice" })` will
  give you "Janet is nice".
- `(indoc name string)` lets you do indented heredocs. Janet uses any number of
  backticks to denote multiline strings. If you use normal soft quotes,
  everything will be on the same line.
- `(pathcat component component...)` joins together its arguments into a Unix
  path.
- `(argcat component component...)` joins its arguments with spaces, to let you
  easily construct commands. `(argcat)` and `(pathcat)` are neater than
  `(string)` when you use vars.
- `(fields)` changes a whitespace-separated string into an array of strings.
- `(run-cmd)` runs the given command (specified as a single string), and gives
  you either a string of its stdout or an `(error)` of its stderr. Far too basic
  to deal with pipes, but still useful.

### Dynamic Bindings

- `*gurp-config-root*` is bound to the directory which contains your
  top-level host config file.
