## How to Configure a machine

> This is a work-in-progress and subject to (possibly complete) change.

Config is written in [Janet](https://janet-lang.org) which is a Lisp (or not,
depending on how much of a Lisp nerd you are). This means you can write
configuration as code, rather than configuration as configuration file. You can
do anything your Janet chops allow, provided you end up calling the resource
definition functions. A couple of things to bear in mind.

- Janet does not hoist functions. You can't refer to something until you've
  defined it.
- The Janet is compiled into a definition, which is then used to assert state.
  That means you could check some condition which is true when Gurp compiles
  your Janet, but not true when Gurp tries to assert state. For instance, your
  code could check for the presence of something it creates itself.
- Yes, parentheses. Deal with it.

## Host Definition

A host definition starts with a `host` definition. Obviously, I suppose. This
definition references three modules.

```janet
(use basenode)
(use user-tools)
(use security)

(host "example-host"
      (basenode)
      (user-tools)
      (security))
```

The host name isn't used for anything other than logging. It won't change the
actual hostname.

Note that we didn't need to `(include)` or `(use)` any library file to get
access to `(host)`. Gurp has a hardcoded library file which is automatically
injects at the top of your code, and it contains the `(host)` macro. If you run
Gurp with `--dump-config` you will see the full augmented file.

## Roles

A host needs one or more roles. If you want, you can put the roles in the same
file as the host definition, but you'll need to put them above the `(host)`
because of the warning I gave you earlier.

Typically you'll put roles in their own files, as you would with any other
config management tool. They follow the normal Janet importing rules, so the
`basenode` role could be:

- `basenode.janet`
- `basenode/init.janet` (which could `use` other files in that directory)
- `basenode.jimage` (if you don't know what this is, you don't need to)

The include path is automagically manipulated so roles will be found at the same
level as your host definition file. If you want to reference them elsewhere,
you'll need to `(use)` them.

```janet
(role "my-role"
      (pkg/ensure "ooce/developer/rust")
      (pkg/remove "ooce/developer/go")
      (file/ensure "/etc/application/config.txt"
                   :label "app-config"
                   :owner "root"
                   :content (string "config values for " (this-host)))
      (directory/ensure "/etc/application"
                        :owner :/my-role/file/app-config/owner
                        :group "engineering"
                        :mode "0750"))
```

## Resources

Resources are all defined in the same way. `(resource-type/ensure)` or
`(resource-type/remove)`, a string name, and pairs of symbol keys and string
values. The `:keyword "string"` format is the way we idiomatically define
key-value pairs in Janet. You can't use commas: they mean something.

Note that the `:owner` of the directory is a Janet keyword. This is a reference
which will be followed, and make the owner of the directory the same as that of
the file. References can refer to other references, and Gurp is able to detect
unresolvable loops and dangling references.

Typically a reference takes the form
`/role-name/resource-type/resource-name/resource-property`. Internally every
resource gets an ID constructed in that way, and you will see them in execution
output. But, the slash-separation could be confusing when referring to file
paths, so Gurp converts slashes to underscores in the id resource property name.
You can use that pattern in your references, but it's usually clearer to follow
the above example and add a `:label`, then refer to that.

There's even a convenience function `(this)`, which lets you refer to a resource
in the same role by writing `(this :file :app-config :owner)`.

We have dynamically constructed a value for the content of the file. `(string)`
concatenates its arguments, and `(this-host)` is a Gurp builtin which expands to
the name you set in your `(host)` definition.

## Variables

Because it aims to be as stupid and unsophisticated as possible, Gurp does not
have attribute hierarchies or any kind of inbuilt variable management. It's up
to you to use Janet. Use `(let)`, use `(def)`, use `(var)`, use prototyped
tables, structs, arrays, tuples, whatever works for you.

First, a vars file with a struct.

```janet
# vars.janet
(def packages
    { :editors ["vim" "neovim" "helix"]
      :languages ["rust" "ruby33"]})
```

```janet
# host.janet
(import "./vars")

(role "my-role"
  (each pkg (get vars/packages editors)
    (pkg/ensure (string "/ooce/editor/" pkg))))
```

Next, a vars file where we use the built in `(this-host-k)` macro to get our
value. If you prefer, you can make the keys strings and use `(this-host)`.

```janet
# vars.janet
(def packages
  {:host-a ["vim" "ruby"]
   :host-b ["helix" "rust"]})
```

```janet
# host.janet
(import "./vars")

(role "my-role"
  (loop [pkg :in (get vars/packages (this-host))]
    (pkg/ensure (string "/ooce/editor/" pkg))))
```

Now, lexical scoping with a Janet `let`.

```janet
(let [log_dir "/var/log"]
  (directory/ensure log_dir
                    :mode "0775"
                    :group "loggers")

  (cron/ensure "log-rotate"
               :minute 0
               :hour 0
               :command (argcat "/bin/log-rotator" log_dir)))
```

"Variables" don't have to be static variables. They can be helper functions, or
the result of some action bound in a `(def)`. You could even stick a `(macro)`
or two in there.

Isn't that more civilised than shoehorning weirdness into YAML and counting
indents?

## Default/Fallback Values

Some resources have default values. We do this by means of
[Janet table prototypes](https://janet-lang.org/docs/prototypes.html). At the
moment, these are hardcoded into Gurp, but by the time we're finished you will
also be able to supply your own.

You can see the default values by running `gurp show defaults`.

## Comments

Gurp doesn't provide any way to comment things, but you can of course use Janet
comments, which are prefixed with a `#`, or enclosed in a `(comment)`.

## Sections

The Gurp library includes a `(section)` macro. It does nothing, but allows you
to visually associate related resources.

```janet
(file/ensure "/some/file/not-to-do-with-logging")

(section "logging-setup"
    # in which we set up our log rotation stuff
    (directory/ensure "/var/log/dir" :mode "0775" :group "loggers")
    (cron/ensure "log-rotate"
                 :minute 0
                 :hour 0
                 :command (argcat "/bin/log-rotator" "/var/log/dir")))

(directory/ensure "/some/dir/also-not-to-do-with-logging")
```

I should have mentioned `(argcat)` earlier. It's another Gurp macro which joins
its arguments with spaces. It's nicer than using `(string)` and having to
remember to add leading and trailing spaces, and less typing than
`(string/format)`. There's also `(pathcat)`, which joins with slashes.
