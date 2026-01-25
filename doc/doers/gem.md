# gem

Install and uninstall Ruby gems.

## Resouce Name

Gem name (`:string`)

## gem/ensure

```janet
(gem/ensure "wavefront-cli")
```

```janet
(gem/ensure "my-gem"
            :version "1.2.3"
            :source "https://my-gem-repo.com")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gem-path` | `string` | Path to gem executable other than /opt/ooce/bin/gem |  |
| `:source` | `string` | Source other than RubyGems. Can contain tokens and usernames |  |
| `:version` | `string` | Gem version |  |

## gem/remove

```janet
(gem/remove "webscale")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:gem-path:` | `string` | Path to gem executable other than /opt/ooce/bin/gem |  |
| `:version` | `string` | Gem version |  |

