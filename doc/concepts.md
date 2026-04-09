# Gurp Concepts

## What is Gurp?

Gurp is a config management tool squarely targeted at
[illumos](https://illumos.org). It is developed on [OmniOS](https://omnios.org)
but should work happily on other distributions such as OpenIndiana and Tribblix.

## The Configuration Language

To use Gurp, you define the state of your system using a DSL based around
[the Janet programming language](https://janet-lang.org). Janet is or is not a
Lisp, depending on your point of view, with a Clojure-inspired syntax. So as
well as having lots of parentheses (though not as many as you might expect) it
also has square and curly brackets, making it nicer to read.

### Variables

Because Janet is a real programming language, Gurp has no horrible hacks to
incorporate logic, variables, or functions. There is also no formal concept of
anything like Hiera, or Chef's attribute precedence. Runtime variables are
normal Janet variables, and your code is normal Janet code. However, since
[Janet tables support prototypes](https://janet-lang.org/docs/prototypes.html)
you can quite easily construct an attribute hierarchy if you need one.

## Resources, Doers, and Helpers

Like most similar tools, Gurp works on a concept of "resources". A resource
might be a file, a Unix user, a service, or the system scheduler. A user can
`ensure` the state of a resource like so:

```janet
(file/ensure "/my/new/file"
  :owner "root"
  :group "daemon"
  :content "important data")
```

The properties of the resource are defined in key-value pairs. Note that in
Janet, keywords are **prefixed** with a `:`.

You can also guarantee that a resource is removed, if it makes sense for that
resource to be removed. (For instance, you can `ensure` which class the
scheduler uses, but you can't remove it.)

```janet
(file/remove "/that/old/file")
```

Those file resources are handled by what Gurp calls the file "doer". Most other
things would call this a "provider", but what does a provider actually provide?
Doers, clearly, are things that do the things.

`gurp doers` lists the built-in doers, with brief descriptions.

Some doers, most notably `zone`, also have what we call "helpers".

```janet
(zone/ensure "bhyve-zone"
             :brand "bhyve"
             :autoboot false
             :image "/var/tmp/noble-server-cloudimg-amd64.img.raw"
             (zone/network "bhyve0"
                           :allowed-address "192.168.1.102/24"
                           :global-nic "auto")
             (zone/bhyve
               :ram "4G"
               :vcpus 4
               :boot-volume "tank/bhyve/test"
               :cloudinit-struct {:network {:version 2}})

             :dns {:domain "lan.id264.net"
                   :nameservers ["192.168.1.53"
                                 "192.168.1.1"]})
```

Here we see two helpers: `zone/network` and `zone/bhyve`, and lots of properties
Some resource properties are mandatory, others are optional:
[Gurp's documentation](https://github.com/snltd/gurp/tree/main/doc/doers)
explains them all, and that same documentation is built into Gurp, accessed by
`gurp describe zone`, or whichever other doer you're interested in.

## Hosts and Roles

Unless you are using `gurp apply --exec`, Gurp config must be wrapped in a
`host` definition. A host can include roles, which generally live in their own
files and are brought into scope with `use`.

```janet
(use "roles/security")
(use "roles/tools")

(host "example"
  (security)
  (tools))
```

```janet
(role tools
  (pkg/ensure "ooce/developer/rust")
  (pkg/ensure "ooce/editor/helix"))
```

## Resource IDs

When Gurp compiles the user's config, it gives each resource an ID string. IDs
follow the general format

```
/role/resource-type/resource-name
```

If your resource is not part of a role, the first part is `NO-ROLE`. If your
resource name contains slashes, as file and directory resources always will,
they are converted to underscores. This can become unwieldy, so all resources
accept a `:label` property. If you use this, the ID is of the form

```
/role/resource-type/label
```

Gurp does not allow duplicate resource IDs.

Resource IDs often show up in debug logs, and you use them to filter resources
wlth `gurp apply --only <REGEX>`. Labels also let one Gurp resource refer to another.

## References

One Gurp resource may refer to another:

```janet
(role example
  (directory/ensure "/dir"
    :label "ref"
    :owner "example")

  (file/ensure "/dir/file1"
    :label "file1"
    :owner :/example/file/file1))

  (file/ensure "/dir/file2"
    :owner (this :directory :ref)))
```

`file1` users a literal keyword to point to the directory resource, and `file2`
uses the `this` function to refer to a directory called `ref` inside the same
role.

References may be chained, and Gurp will spot unresolved or circular references.

You cannot refer a property of a resource not managed by Gurp, or a property of
a Gurp-managed resource not defined in the config.

## Ordering and Dependencies

Gurp does not have explicit ordering of resources. You cannot say "this file
depends on this directory". Instead, Gurp applies all resources of the same type
in a particular order. So far, this has been fine...

## Running Arbitrary Commands

YOU CAN'T. Gurp has no "command" doer. If you want to run shell scripts, go and
run shell scripts. If it's important enough to be done dozens of times a day on
a production machine, then it's important enough to be done in a proper
programming language, with tests. Open an issue, or submit a PR.
