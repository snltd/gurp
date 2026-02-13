# apk

Install and uninstall APK packages. Only valid in an Alpine LX zone.

## Resource Name

Package name (`:string`)

## apk/ensure

```janet
(apk/ensure "rust")
```

### Mandatory Properties

None

### Optional Properties

None

## apk/remove

```janet
(apk/remove "go")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- Only adds and removes packages. You cannot specify or pin package versions.
- The package database is refreshed prior to an install
