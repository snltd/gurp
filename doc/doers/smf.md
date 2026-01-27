# smf

Create and install a manifest for an SMF service.

## Resource Name

Short name of service. Not used internally (`:string`)

## smf/ensure

```janet
(smf/ensure "example"
            :description "Run example program"
            :fmri "snltd/example"
            (smf/dependency "dependency1"
                            :fmri "svc://example/service1:default")
            (smf/dependency "dependency2"
                            :grouping "optional-all"
                            :restart-on "error"
                            :fmri "svc://example/service2:default")
            (smf/method "start"
                        :exec "/opt/site/lib/smf/method/example.sh"
                        :user "example"
                        :group "daemon"
                        :privileges ["basic"
                                     "file_dac_search"
                                     "sys_admin"
                                     "proc_owner"
                                     "proc_zone"])
            :property-groups {:application "application"}
            :properties {:application/datadir "/data"})
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:fmri` | `string` | Service FMRI |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:default-enabled` | `boolean` | Start the service when the manifest installs | `true` |
| `:dependencies` | `array` | See 'smf-dependency' |  |
| `:dependents` | `array` | See 'smf-dependent' |  |
| `:description` | `string` | What the service does |  |
| `:duration` | `string` | Use this to specify 'transient' or 'wait' services |  |
| `:properties` | `struct table` | Create/set properties.(:keyword :string|:boolean|:number) |  |
| `:property-groups` | `struct table` | Create property groups. Key is the name, value is the type |  |
| `:refresh-method` | `struct table` | See 'smf-method' |  |
| `:restart-method` | `struct table` | See 'smf-method' |  |
| `:single-instance` | `boolean` | Is this a single-instance service | `true` |
| `:start-method` | `struct table` | See 'smf-method' |  |
| `:stop-method` | `struct table` | See 'smf-method' | `{:exec ":kill" :timeout 10}` |

## smf/remove

```janet
(smf/remove "unwanted/service")
```

### Mandatory Properties

None

### Optional Properties

None

# smf/dependency

Defines a dependency of an SMF service, inside an                            smf resource.

## Sub-Resource Name

This sub-resource does not accept a name

```janet
(smf/dependency "example-1"
                :grouping "optional-all"
                :restart-on "error"
                :fmri "svc://example/service-1:default")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:fmri` | `string` | Dependency FMRI |  |
| `:name` | `string` | Convenient name for dependency, derived from resource name |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:grouping` | `string` | Which dependencies are required by this service | `"require_all"` |
| `:restart-on` | `string` | Policy for restarting this service if dependency restarts | `"none"` |
| `:type` | `string` | Type of dependency | `"service"` |


# smf/dependent

Defines a dependent of an SMF service, inside an                            smf resource.

## Sub-Resource Name

This sub-resource does not accept a name

```janet
(smf/dependent "example-1"
               :grouping "optional-all"
               :restart-on "error"
               :fmri "svc://example/service-1:default")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:fmri` | `string` | Dependent FMRI |  |
| `:name` | `string` | Convenient name for dependent, derived from resource name |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:grouping` | `string` | Which dependencies are required by this service | `"require_all"` |
| `:restart-on` | `string` | Policy for restarting this service if dependent restarts | `"none"` |
| `:type` | `string` | Type of dependent | `"service"` |


# smf/method

Defines an SMF method to launch a service state

## Sub-Resource Name

One of "start", "stop", "refresh", "reload" (`:string`)

```janet
(smf/method "start"
            :exec "/opt/site/lib/smf/method/example.sh"
            :user "example"
            :group "daemon"
            :privileges ["basic"
                         "file_dac_search"
                         "sys_admin"
                         "proc_owner"
                         "proc_zone"])
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:exec` | `string` | Method or command to execute |  |
| `:timeout` | `number` | Seconds until method times out | `60` |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:environment` | `struct table` | Environment variables set inside context |  |
| `:group` | `string` | Group the method runs as |  |
| `:privileges` | `tuple` | Privileges the method has. Use ! to remove them |  |
| `:user` | `string` | User the method runs as |  |

