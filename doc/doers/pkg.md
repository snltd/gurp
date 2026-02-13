# pkg

Install and uninstall pkg(5) packages.

## Resource Name

Package name, of the form ooce/editor/helix (`:string`)

## pkg/ensure

```janet
(pkg/ensure "ooce/developer/rust")
```

### Mandatory Properties

None

### Optional Properties

None

## pkg/remove

```janet
(pkg/remove "ooce/developer/go")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- You specify packages by name, so `ooce/editor/helix` rather than `pkg://sysdef/ooce/editor/helix@25.1-151052.0:20250108t110907Z`. This means you can't request specific versions. It isn't remotely smart and can't understand that `helix` is `ooce/editor/helix`.
- Gurp does not support package mediators.
- If Gurp runs with `--noop`, `pkg` will be executed with the `-n` flag. Therefore it can cause a noop run to fail.
- These limitations mean we can ensure all packages are installed with just two calls to `pkg(1)`. This is a huge speed win over dealing with each package individually
