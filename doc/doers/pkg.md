# pkg

Install and uninstall pkg(5) packages.

## pkg/ensure

### Resource Name

Package name, of the form ooce/editor/helix (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(pkg/ensure "ooce/developer/rust")
```

## pkg/remove

### Resource Name

Package name, of the form ooce/editor/helix (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(pkg/remove "ooce/developer/go")
```

## Notes

- You specify packages by name, so `ooce/editor/helix` rather than `pkg://sysdef/ooce/editor/helix@25.1-151052.0:20250108t110907Z`. This means you can't request specific versions. It isn't remotely smart and can't understand that `helix` is `ooce/editor/helix`.
- Gurp does not support package mediators.
- If Gurp runs with `--noop`, `pkg` will be executed with the `-n` flag. Therefore it can cause a noop run to fail.
- These limitations mean we can ensure all packages are installed with just two calls to `pkg(1)`. This is a huge speed win over dealing with each package individually
