# apk

Install and uninstall APK packages. Only valid in an Alpine LX zone.

## apk/ensure

### Resource Name

Package name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(apk/ensure "rust")
```

## apk/remove

### Resource Name

Package name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(apk/remove "go")
```

## Notes

- Only adds and removes packages. You cannot specify or pin package versions.
- The package database is refreshed prior to an install
