# smf

Create and install a manifest for an SMF service.

## smf/ensure

```janet
(smf/ensure "telegraf"
            :description "Run Telegraf agent"
            :fmri "sysdef/telegraf"
            (smf/dependency "svc1"
                            :fmri "svc://example/service1:default")
            (smf/dependency "svc2"
                            :grouping "optional-all"
                            :restart-on "error"
                            :fmri "svc://example/service2:default")
            (smf/method "start"
                        :exec "/opt/site/lib/smf/method/telegraf.sh"
                        :user "telegraf"
                        :group "daemon"
                        :privileges ["basic" "file_dac_search" "sys_admin"
                                     "proc_owner" "proc_zone"])
            :property-groups {:application "application"}
            :properties {:application/datadir "/data"})
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| fmri | `string` | Service FMRI |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| default-enabled | `boolean` | Start the service when the manifest installs | true |
| dependencies | `array` | See 'smf-dependency' |  |
| dependents | `array` | See 'smf-dependent' |  |
| description | `string` | What the service does |  |
| duration | `string` | Use this to specify 'transient' or 'wait' services |  |
| properties | `struct table` | Create/set properties.(:keyword :string|:boolean|:number) |  |
| property-groups | `struct table` | Create property groups. Key is the name, value is the type |  |
| refresh-method | `struct table` | See 'smf-method' |  |
| restart-method | `struct table` | See 'smf-method' |  |
| single-instance | `boolean` | Is this a single-instance service | true |
| start-method | `struct table` | See 'smf-method' |  |
| stop-method | `struct table` | See 'smf-method' | <struct 0x000001D76B78> |

## smf/remove

```janet
(smf/remove "some/unwanted/service")
```

### Mandatory Properties

None

### Optional Properties

None

There is no smf/remove.