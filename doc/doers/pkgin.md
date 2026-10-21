# pkgin

Install and uninstall pkgin packages. Only valid in a pkgsrc zone.

## pkgin/ensure

### Resource Name

Package name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(pkgin/ensure "rust")
```

## pkgin/remove

### Resource Name

Package name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(pkgin/remove "go")
```

## Notes

- You specify pkgs by name, so `openssl` rather than `openssl-3.3.2`. This means you can't request specific versions.
