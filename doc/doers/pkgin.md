# pkgin


Install and uninstall pkgin packages. Only valid in a pkgsrc zone.

### Resource Name

Package name [`:string`]

## pkgin/ensure


### Mandatory properties

None

### Optional properties

None

## pkgin/remove


### Mandatory properties

None

### Optional properties

None

## Notes

- You specify pkgs by name, so `openssl` rather than `openssl-3.3.2`. This means you can't request specific versions.


### Examples

```janet
(pkgin/ensure "rust")
```


```janet
(pkgin/remove "go")
```
