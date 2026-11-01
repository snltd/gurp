# system-cert


Add and remove system TLS certificates

### Resource Name

File certificate will have in /etc/ssl/certs [`:string`]

## system-cert/ensure


### Mandatory properties

None

### Optional properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:checksum` | `struct` | See file/checksum. |  |
| `:content` | `string` | Use this literal string as the cert. |  |
| `:from-url` | `string` | Fetch cert from the given URL. |  |
| `:from` | `string` | Copy cert content from this file. If relative, looks in `../files`. |  |
| `:url-is-server` | `boolean` | Used internally to identify Gurp server URLs. |  |

## system-cert/remove


### Mandatory properties

None

### Optional properties

None

## Notes

- Does not generate certs: just copies them to the system directory and re-hashes it.
- If a `:from` path is relative, Gurp will fully qualify it using the same rules as the `file` doer.


### Examples

```janet
(system-cert/ensure "from-file"
                    :from "ca/example")
```
```janet
(system-cert/ensure "from-url"
                    :from-url "https://cert-service/api"
                    (file/checksum
                      :algorithm "sha256"
                      :with-suffix ".sha256"))
```


```janet
(system-cert/remove "unwanted-cert")
```
