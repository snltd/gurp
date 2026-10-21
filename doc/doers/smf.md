# smf

Create and install a manifest for an SMF service.

## smf/ensure

### Resource Name

Short name of service. Not used internally (`:string`)

### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:default-enabled` | `boolean` | Start the service when the manifest installs. | `true` |
| `:fmri` | `string` | Service FMRI. |  |
| `:single-instance` | `boolean` | Is this a single-instance service. | `true` |
| `:stop-method` | `struct table` | See 'smf-method'. | `<struct 0x00000149C578>` |

### Optional properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:dependency` | `array` | See 'smf-dependency'. |  |
| `:dependent` | `array` | See 'smf-dependent'. |  |
| `:description` | `string` | What the service does. |  |
| `:duration` | `string` | Use this to specify 'transient' or 'wait' services. |  |
| `:properties` | `struct table` | Create/set properties. (`:keyword :string|:boolean|:number`). |  |
| `:property-groups` | `struct table` | Create property groups. Key is the name, value is the type. |  |
| `:refresh-method` | `struct table` | See 'smf-method'. |  |
| `:restart-method` | `struct table` | See 'smf-method'. |  |
| `:start-method` | `struct table` | See 'smf-method'. |  |

### Examples

```janet
(smf/ensure "example"
            :description "Run example program"
            :fmri "snltd/example"
            :duration "child"
            (smf/dependency "dependency1"
                            :fmri "svc:/milestone/name-services:default")
            (smf/dependency "dependency2"
                            :grouping "optional_all"
                            :restart-on "error"
                            :fmri "svc:/system/pkgserv:default")
            (smf/method "start"
                        :exec "/app/method.sh"
                        :user "appuser"
                        :group "daemon"
                        :privileges ["basic"
                                     "!file_dac_search"])
            :property-groups {:application "application"
                              :other_group "framework"}
            :properties {:application/port 8080
                         :application/ssl true
                         :other_group/other_prop "abc123"})
```

## smf/remove

### Resource Name

Short name of service. Not used internally (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(smf/remove "unwanted/service")
```

## Notes

- Generated manifests are written to `/opt/site/lib/smf/manifest`, and the output of subsequent runs is compared to the existing reference. If the file has changed, the service is re-created. We do it this way because it is not possible to compare a running service with a generated manifest.
- smf/remove stops the service and deletes it from the SMF registry.
