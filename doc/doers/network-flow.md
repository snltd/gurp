# network-flow

Manage network flows via flowadm.

## Resouce Name

Name of flow. Must be unique (`:string`)

## network-flow/ensure

```janet
(network-flow/ensure "tls-throttle"
                       :link "vnic1"
                       :protocol "tcp"
                       :remote-ip "203.0.113.4"
                       :remote-port 443
                       :maxbw "10M")
```

```janet
(network-flow/ensure "ssh-flow"
                     :link "vnic0"
                     :protocol "tcp"
                     :local-port 22
                     :maxbw "1M")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:link` | `string` | NIC/VNIC to which flow applies |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:dsfield` | `string` | With optional :mask |  |
| `:local-ip` | `string` | Local IP address for flow, optional /mask |  |
| `:local-port` | `number` | Local port of flow |  |
| `:maxbw` | `string` | Maximum duplex bandidth, with K, M or G suffix |  |
| `:priority` | `string` | Priority of link: high, medium, low |  |
| `:protocol` | `string` | Flow protocol |  |
| `:remote-ip` | `string` | RemoteIP address for flow, optional /mask |  |
| `:remote-port` | `number` | Remote port of flow |  |

## network-flow/remove

```janet
(network-flow/remove "unwanted")
```

### Mandatory Properties

None

### Optional Properties

None

