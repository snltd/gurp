# system-cert

Add and remove system TLS certificates

## Resource Name

File certificate will have in /etc/ssl/certs (`:string`)

## system-cert/ensure

```janet
(system-cert/ensure "from-file"
  :from "/dir/ca/example")
```

```janet
(system-cert/ensure "from-url"
  :from-url "https://cert-service/api")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:content` | `string` | Use this literal string as the cert |  |
| `:from-url` | `string` | Fetch cert from the given URL |  |
| `:from` | `string` | Copy cert content from this file. If relative, looks in ../files |  |

## system-cert/remove

```janet
(system-cert/remove "unwanted-cert")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- Does not generate certs: just copies them to the system directory and re-hashes it.
- If a `:from` path is relative, Gurp will fully qualify it using the same rules as the `file` doer.
