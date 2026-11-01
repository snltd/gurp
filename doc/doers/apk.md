# apk


Install and uninstall APK packages. Only valid in an Alpine LX zone.

### Resource Name

Package name [`:string`]

## apk/ensure


### Mandatory properties

None

### Optional properties

None

## apk/remove


### Mandatory properties

None

### Optional properties

None

## Notes

- Only adds and removes packages. You cannot specify or pin package versions.
- The package database is refreshed prior to an install.


### Examples

```janet
(apk/ensure "rust")
```


```janet
(apk/remove "go")
```
