# Adding a New Doer

## Front-End

Gurp's front-end, like the user's config definitions, is written in
[Janet](https://janet-lang.org). You don't need to know much Janet to add a new
doer.

The front-end's job is to take a bunch of user resource descriptions and turn
them into a single Janet structure called the "collector". To add a new doer,
you first have to define the properties a resource expects to have, and to write
a function to turn them into a structure of the expected type. Gurp provides the
logic to take care of most of that for you.

### Create a Doer Description

Create a new Janet doer description in `janet/src/doers`. Find something of a
reasonably similar shape, copy it and edit the copy. The files are pretty
self-explanatory,

Make sure you `(use ../lib)` and `(import ../collector)` at the top of the file.

You must define (with `def` statements):

- `doer`: The name of the doer, as a keyword. This will be used as the doer
  namespace, and should match the filename. e.g. the `directory` doer is in
  `doers/directory.janet`, has a `doer` binding of `:directory`, and its
  functions are accessed like `directory/ensure`.
- `description`: A description. This is used by the `gurp doers` and
  `gurp describe`.
- `name-is`: A description of the resource's name property, If there isn't one,
  say so.
- `mandatory-props-ensure`: A struct of properties where they key is the
  property name as a keyword, and the value is a struct with keys `:types` (a
  tuple of Janet types the property may be) and `:help`, a brief (string)
  description of that property. Gurp will throw an error if the user does not
  provide each of these properties in a resource `ensure` definition, or if any
  of them are of the wrong types. (It does this _after_ merging user-supplied
  properties with any default ones.) The `:help` is shown by the `describe`
  command.

  Lisp favours kebab-case, so use that for your property names,

- `optional-props-ensure`: A struct just like `:mandatory-props-ensure`. Gurp
  will allow these properties to be set in a resource `ensure` definition, and
  will error if they are set but with the wrong type. But, it will not complain
  if the property is omitted. If the user supplies any properties which are not
  defined as mandatory or optional, Gurp will error. (`:label` is a special
  property: every resource accepts this implicitly.)
- `mandatory-props-remove`: Just like `mandatory-props-ensure`, but checked for
  a `remove` resource. Usually empty.
- `optional-props-remove`: Just like `mandatory-props-ensure`, but checked for a
  `remove` resource. Usually empty.
- `defaults-ensure`: A struct of property names and values which will be merged
  with user `ensure` input prior to property validation. User-supplied values
  take priority.
- `defaults-remove`: Just like `defaults-ensure`, but for `remove` resources.

Now you need an `ensure` function, and probably a `remove`. Here is the simplest
(and most common) example.

```janet
(defn ensure
  "Given a name and spec, put a struct in the collector"
  [name & spec]
  (collector/push :ensure doer (make-ensure-resource)))
```

This will take the user's specification and name, check properties against the
expected values you defined earlier, and put an `ensure` resource in the
collector struct, which aggregates resources.

If you need to perform some actions on the spec, perhaps to validate it or to
modify properties, this is a good pattern to follow. It is pretty much what you
see if you expand the `make-ensure-resource` macro.

```janet
(defn ensure
  [name & spec]
  (def spec-table (struct/to-table (make-spec-struct ;spec)))

  # modify or check spec-table as required

  (def all-specs (spec-with-defaults defaults-ensure spec-table))
  (def safe-specs (checked-spec all-specs
                                mandatory-props-ensure
                                optional-props-ensure))

  (collector/push :ensure doer (spec->resource doer name safe-specs)))
```

It is best not to validate user input too much. If Gurp will end up calling an
OS tool which accepts only some range of a particular property, let that tool
error. The valid range may change, and we don't want to try to keep Gurp in-sync
with a whole operating system.

### Make Some Tests

You should, of course, test `ensure` and `remove` produce the correct output,
particularly if you are following the second pattern and applying custom logic.

Make a file in `janet/test/doers`, with the same name as your doer definition.
We use [Judge](https://github.com/ianthehenry/judge) for testing, so
`(use judge)`. It is customary to check the resource goes into the collector, so
`(use ../../src/collector)` as well. Finally,
`(import ../../src/doers/<filename>)` to access your doer file.

Tests are defined inside a `(deftest)`. Here's a sample:

```janet
(deftest bridge
  (setdyn :role-dyn "test-role")
  (set *collector* (new-collector))

  (bridge/ensure "test_a")

  (bridge/ensure "test_b"
                 :links ["stub0" "vnic0" "e1000g0"]
                 :priority 4096
                 :max-age 30)

  (bridge/remove "test_c")
```

Run `judge <filename>` and Judge will show you what your code produces:

```janet
(test *collector*
  @{:ensure @{:bridge @[{:_id "/test-role/bridge/test_a"
                         :name "test_a"
                         :role "test-role"}
                        {:_id "/test-role/bridge/test_b"
                         :links ["stub0" "vnic0" "e1000g0"]
                         :max-age 30
                         :name "test_b"
                         :priority 4096
                         :role "test-role"}]}
    :remove @{:bridge @[{:_id "/test-role/bridge/test_c"
                         :name "test_c"
                         :role "test-role"}]}}))
```

Note that the `;_id` and `:role` are automatically filled in. If the output is
what you think it should be, run `judge -a <filename>` and Judge will rewrite
the file with the expected test output in it.

It is good to test what happens with bad input. For this, use `(test-error)`.
e.g.

```
(deftest "apk-error"
  (test-error
    (apk/ensure "gurp" :oops "bad property"))
```

Again, use `judge -a` to commit the values.

### Link the Doer

Add a line for your doer in `janet/src/doers.janet`.

```janet
(import ./doers/thing :export true)
```

If you don't do this, the functions you defined will not be found.

Also add a line to the `list-doers` `descriptions` in
`janet/src/commands.janet`. Without this, your doer will not be listed when the
user runs `gurp doers`.

```janet
["thing" thing/description]
```

That's the front-end part taken care of.

## Back-End

Now we have to write some Rust. This is more involved than the front-end work,
and mostly takes place in `doers/src`.

We will assume your doer has `ensure` and `remove`. Obviously, if you have no
remove, simply omit those parts.

### Write Doer

Each doer has its own file. If your doer is called `thing`, add `pub mod thing;`
to `doers/src/lib.rs` and open a new file `doers/src/thing.rs`.

When Gurp runs, it internally turns the collector Janet struct into a single
JSON object.

The first thing our doer will do is deserialize the relevant parts of that
object into structs, using Serde. For our thing doer we'd call these
`GurpThingEnsure` and `GurpThingRemove`, and they would look like this:

```rust
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct GurpThingEnsure{
  #[serde(rename = "_id")]
  pub id: String,
  pub name: String,
}
```

The `serde(rename_all)` automatically turns all your `kebab-case` property names
into `snake_case`.

Any properties you defined in the doer need to be added to this struct. Optional
properties likely are `Option<>` types.

Gurp expects those structs to implement `apply`:

```rust
impl GurpThingEnsure {
  pub fn apply(&self, opts: &ApplyOpts) -> anyhow::Result<ApplySummary> {
  // The logic that aligns the system with the state defined in self. Typically:
  //
  // if resource_exists {
  //   if current_state == desired_state {
  //      Ok(ONE_RESOURCE_NO_CHANGE)
  //   } else {
  //      align_current_state_with_desired_state
  //      Ok(ONE_RESOURCE_ONE_CHANGE)
  // } else {
  //   create_resource_with_desired_state
  //   Ok(ONE_RESOURCE_ONE_CHANGE
  // }
}
```

You usually have a top-level "does thing exist?" function called by the `Ensure`
and `Remove` impls.

### Add Types

In `doers/src/types.rs` add a
`use crate::thing::{GurpThingEnsure, GurpThingRemove};` line. Then add those
types to the `EnsureResources` and `RemoveResources` structs.

### Call Doers

Finally, add two calls to the `apply_resources!` macro in `doers/src/host.rs`.
(One for ensure and one for remove.) Think carefully where it needs to go: any
resource types your resource depends on need to exist.

## Documentation

Describe the doer in `doc/doers.md`. It will closely resemble your doer
definition from the first step. If there is anything a user might not expects,
such unsupported properties or other limitations, note it here.
