## describe

Describe a resource type.

```
Usage: gurp describe [OPTIONS] <RESOURCE>

Arguments:
  <RESOURCE>  Resource type you wish to see described

Options:
  -C, --no-colour  Do not use any ANSI colouring
  -h, --help       Print help
```

The `describe` command is Gurp's online documentation. Its output is generated
from the doer definition files, so it is always up-to-date and correct.

You can `describe` any doer (e.g. `zone`) or sub-resource (e.g. `zone/bhyve`)

```
$ gurp describe -C zone
zone
  Create and destroy zones. Existing zones cannot be modified.

zone/ensure
  name  [:string]  Zone name

Mandatory properties
  brand [:string]  Zone brand

Optional properties
  rctl               [:array]    See zone/rctl
...
$ gurp describe -C zone/rctl
zone/rctl
  Define a resource control when creating a zone.

  name  [:string]  RCTL name

Mandatory properties
  priv   [:string]  rctl privilege
  name   [:string]  private field managed by Gurp
  action [:string]  rctl action
  limit  [:number]  rctl limit value

Optional properties
  None
```

By default `gurp describe` uses ANSI control codes for highlighting. If you
specify `-C`, or direct output in any way, those codes are removed.
