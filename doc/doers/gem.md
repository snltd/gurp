# gem

Install and uninstall Ruby gems.

## gem/ensure

```janet
(gem/ensure "my-gem" :source "https://my-gem-repo.com")
```

```janet
(gem/ensure "wavefront-cli" :version "8.0.1")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| gem-path | `string` | Path to gem executable other than /opt/ooce/bin/gem |  |
| source | `string` | Source other than RubyGems. Can contain tokens and usernames |  |
| version | `string` | Gem version |  |

## gem/remove

```janet
(gem/remove "webscale")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| gem-path: | `string` | Path to gem executable other than /opt/ooce/bin/gem |  |
| version | `string` | Gem version |  |

There is no gem/remove.