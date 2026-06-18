## Janet Helpers

### Functions/Macros

As well as
[all of Janet's standard library](https://janet-lang.org/api/index.html), `gurp`
gives you a few convenience functions and macros.

- `(host)` is the top-level form for a Gurp configuration. Configuration will
  only be compiled and applied if it is inside a `host` definition, Hosts can
  contain raw resources (e.g. `(pkg/ensure)`) or roles.
- `(role)` is a container for multiple Gurp resources. Once defined, roles must
  be included in a host config by executing them like `(this)`). Roles can be
  written to accept parameters, in which case they are called as `(example-role
  :param1 "value1 :param2 456)`. Parameters end up in a role-scoped struct
  called `role-params`. When writing a role, you can force it to error if
  certain params are not supplied by having `(require :param1 :param2)` as its
  first form.
- `(this-host)` returns the name of the host as defined in the top level
  `(host)` declaration, **not** the result of `(uname -n)`.
- `(this-host-k)` is the output of `(this-host)`, but as a Janet keyword, which
  makes it more convenient for struct/table lookups.
- `(this-role)` returns the name of the enclosing role.
- `(this-role-k)` as above, but a keyword.
- `(this resource-type resource-name resource-property)` is a convenient way to
  return a reference to a resource in the current role. For example,
  `(this :user :rob :uid)`.
- `(template-out template var-struct)` takes a template string and a struct or
  table, and replaces references in the template with their corresponding values
  in the struct. For instance, `(template-out "{{ prog }} is {{ thing }}. I like
  {{ prog }}" {:prog "Janet" :thing "nice" })` will give you "Janet is nice. I
  like Janet.".
- `(indoc string)` lets you do indented heredocs. Janet uses any number
  of backticks to denote multiline strings. If you use normal soft quotes,
  everything will be on the same line.
- `(section)` does nothing at all, but can be useful to divide your config into,
  well, sections.
- `(pathcat component component ...)` joins together its arguments into a
  fully-qualified Unix path.
- `(argcat component component ...)` joins its arguments with spaces, to let you
  easily construct commands. `(argcat)` and `(pathcat)` are neater than
  `(string)` or `(string/format)` when you use vars.
- `(zfscat pool component ...)` joins its components to make a ZFS dataset name.
- `(parent file)` gives you the parent of `file`.
- `(fields)` changes a whitespace-separated string into an array of strings.
- `(labelise token ...)` turns tokens into a string which is safe to use as a
  resource `:label`.
- `(run-cmd "command")` runs the given command (specified as a single string),
  and gives you either a string of its stdout or an `(error)` of its stderr. Far
  too basic to deal with pipes, but still useful.
- `(config-file)` returns the fully-qualified path of a relative file path. Used
  internally by the `file` doer, but useful on its own.
- `(cron-minutes-from-name seed-string interval)` is used to generate
  minutes-past-the-hour lists to run Gurp (or anything else) periodically from
  cron. It uses `seed-string` to calculate a hash, so different hosts will run
  at different times. `interval` must be a divisor of 60.
- `(repeated-line-file format-string values)` returns a string, for use as a
  config file, where each of `values` is applied to a format string. Good for
  things like automount maps.
- `(compact arr)` returns a new version of an array or tuple with `nil` elements
  removed.
- `(qualified-path?` path)` returns true if the argument looks like a qualified
  path.
- `(cloudinit-meta-data)` returns a cloudinit metadata struct for the given
  hostname.
- `(tabular-output->struct)` Takes as its first argument a string table,
  like the output of `dladm show-link`, or `zoneadm list -cv`, and turns it into
  a struct. The keys of that struct are the column identified by an optional
  second argument, which defaults to 0; the values are structs whose keys are
  the table headers (the first row of the string), lowercased and as keywords,
  and whose values are the values in the table. If any of those values can be
  safely converted into numbers, they are.
- `(recreate?)` used in conjunction with the `:recreate` key in a zone config.
  Write `:recreate (recreate? "my-zone")` and if you run `gurp` with
  `GURP_RECREATE_ZONE=my-zone` , `my-zone` will be recreated.
- `(num-field-sort string)` sorts the lines in `string` based on the assumption
  the first field of each is numeric.
- `(metadata :key value)` add a `:key` `value` pair to a global metadata struct.
  Some keys (currently only `:host` are protected. Attempting to set duplicate
  keys is an error.

### Dynamic Bindings

- `:gurp-config-root` is bound to the directory which contains your top-level
  host config file.
