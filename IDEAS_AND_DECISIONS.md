Because this is a very informal, very erratic side-project, I think it will be
worth recording ideas and trials somewhere. Maybe I should use issues or some
other Github thing, but mainly this is for thinking out loud.

- ~~Use Rust modules to enforce state. Janet lacks some basic things (e.g.
  changing file permissions) and I however great
  [[janet-sh](https://github.com/andrewchambers/janet-sh) might be, I don't want
  to end up shelling out for everything. I also don't want to maintain my own
  fork of Janet.~~ [x]

- Use a complete Rust back-end. State will be described in Janet, and compiled
  into a final structure of some kind by an embedded interpreter. Rust will then
  process said structure, and enforce it. [:check-mark:]

- Bake the necessary Janet function and macro definitions into the Rust
  executable, and inject them into the user's config. The downside of this is
  that it will not be possible for a user to use a Janet REPL to create and
  debug config.

- Add a `--local-libs` option to tell `gurp` (I'm going to have to think of a
  proper name) to use Janet macros and functions stored locally, obviating the
  problem above. Also a command to dump the baked-in ones as a starting point.

- Why did I have a `:vars` structure? Bin that, and let the user import their
  own vars as native Janet, and deal with them however they wish. :check-mark:]

- Maybe one day allow Janet "doers"? I'm not sure how hard/sensible/necessary
  this is, but it might be fun.

- ~~Turning the Janet state definition into properly typed Rust structs is
  tiresome. Have Janet dump is as JSON, and deserialize with Serde.~~ Janet does
  not have JSON support built in, and I don't want Spork getting involved. I
  found a pure Janet module, but it didn't work properly. [x]

- ~~Wrote a function to turn Janet objects into JSON myself, then used
  `serde_json` to turn it into Rust objects.~~ This put me off the generic
  approach. I don't plan to support many resource types, and hand-coding
  Janet->Rust functions for each type means I can have complete control over
  them. I prefer that approach. Ripped out all the JSON stuff. [x]

- Up to now, Janet was making a map, with keys being resource types, and values
  being lists of that resource. This can be troublesome to work with, so I'm
  going to have it produce an array of resources. [:check-mark:]

- Use Janet table prototypes to fill in default values for resources. We'll have
  our own hard-coded value, but let the user supply their own somehow.
  [:check-mark:]

- Add an option to load in a Janet struct rather than making one from Janet
  source. This means you'd only need two files on a box to configure it. Or
  heck, why not have an option to fetch a ready-made Janet struct from a remote
  source? I said I didn't want a server, but pulling pre-built config from some
  HTTP endpoint or Github repo is such an easy win.

- I'm thinking a lot about dependency ordering. Originally I was going to take
  the Ansible approach and have things happen in the order they were defined,
  but as we're already loading everything into structures, it seems a bit silly
  not to use something like
  [petgraph](https://docs.rs/petgraph/latest/petgraph/) to order them properly,
  and catch any circular dependencies. An optional `:before` property is
  necessary, I'm not sure if `:after` is, it might be nice, but it might also
  confuse things. I am, after all, trying to keep this thing as lean and simple
  as possible.

- Starting to wonder if I will need some implicit dependency. Maybe as Janet
  adds resources on to its array, it makes itself dependent on the previous
  element, unless the user say something else? (This means I _do_ need
  `:after`.) There are other more subtle dependencies too. Should you have to
  explicitly say "create `/dir` before `/dir/file`" or should `gurp` work it out
  itself? You can remove some of this by doing all your directories before you
  do any files. And, I think, on a `pkg(5)` system it should be safe to assume
  that packages don't have _any_ dependencies (other than other packages, and
  `pkg(5)` should work that out for us.) Do services last.

- How do we know whether to restart a service? I think they should have a
  `:restart-if` property, with a list of resource IDs. When `gurp` applies
  resources, it should keep a list of things it has changed, and at the end of
  the run, if any of the `:restart-if`s are in there, the service is restarted.

- This is going to run _fast_. I'm still not averse to the CfEngine approach of
  running multiple times, with each run converging towards required state.

- Packages need to be grouped so we only make a single call to the package
  manager. Adding packages is going to be a huge chunk of the execution time.

- I've over-thought dependencies to a point where it's stopped me dead. I'm
  going to go with a very crude approach and once the thing roughly works
  end-to-end I'll revisit it. I feel like I need to see how it behaves, and what
  the final data structures end up looking like.

- ~~Resources should be able to refer to other resources. I'm not sure how far I
  want to go with this. Terraform `data`? Probably not, but you should at least
  be able to say "this file should have the same owner as this file" without
  having to use variables.~~ This is implemented. It catches unresolved and
  circular references, and it all happens in the Janet phase.

- Should we lint user input? Check for required fields? Warn on unknown ones?
  Check for clashing directory/file paths? Clashing resource names?

- As well as `ensure` and `remove`, an `only` or `exactly` command, which
  adds/removes resources of a suitable type until the installed instances match
  the given list.

- Define SMF services in Janet, rather than in XML.

- Handle zones, replacing Oozone. Zones defined, naturally, with Janet.

- A `globals` struct in the Janet, at the same level as `metadata` and
  `resources` that lets the user configure things which will apply to all
  modules. `pkg` opts and stuff.
