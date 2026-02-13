# gem

Install and uninstall Ruby gems.

## Resource Name

Gem name (`:string`)

## gem/ensure

```janet
(gem/ensure "wavefront-cli")
```

```janet
(gem/ensure "my-gem"
            :version "1.2.3"
            :gem-path "/opt/pkgin/bin/gem"
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

## Notes

- Tries to minimise the calls to `gem install` by grouping together installs with similar parameters
- Only version numbers are supported, so `latest` won't work.
- `gem/remove` takes no options, so removes all versions of the given gem.
