# pkgin

Install and uninstall pkgin packages. Only valid in a pkgsrc zone.

## Resource Name

Package name (`:string`)

## pkgin/ensure

```janet
(pkgin/ensure "rust")
```

### Mandatory Properties

None

### Optional Properties

None

## pkgin/remove

```janet
(pkgin/remove "go")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- You specify pkgs by name, so `openssl` rather than `openssl-3.3.2`. This means you can't request specific versions.
