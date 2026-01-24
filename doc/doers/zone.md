# zone

Create and destroy zones. Existing zones cannot be modified.

## Resouce Name

Zone name (`:string`)

## zone/ensure

```janet
(zone/ensure "test-zone-thin"
             (zone/network "test_net0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.33/24"
                           :defrouter "192.168.1.1")
             :brand "lipkg")
```

```janet
(zone/ensure "test-zone-bootstrap-net"
             (zone/network "test_net0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.33/24"
                           :defrouter "192.168.1.1")
             (zone/bootstrap
               :server "gurp.localnet"
               :hostname "test-zone-bootstrap")
             :brand "lipkg")
```

```janet
(zone/ensure "test-lx-zone"
             (zone/network "test_net0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.33/24"
                           :defrouter "192.168.1.1")
             (zone/attr "kernel-ver" :value "4.4")
             :exec-in ["/bin/exec1" "/bin/exec2"]
             :copy-in {"lx-test/f1" "/etc/file1"
                       "lx-test/f2" "/bin/exec2"}
             :brand "lx")
```

```janet
(zone/ensure "test-zone-bhyve"
             :brand "bhyve"
             :autoboot false
             (zone/network "test_net0"
                           :allowed-address "192.168.1.33/24"
                           :global-nic "auto")
             (zone/bhyve
               :ram "3G"
               :vcpus 4
               :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
               :boot-volume "tank/bhyve/test"
               :cloudinit-struct {:network {:version 2}})
             :dns {:domain "lan.id264.net"
                   :nameservers ["192.168.1.53"
                                 "192.168.1.1"]})
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:brand` | `string` | Zone brand |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:attr` | `array` | See zone/attr |  |
| `:autoboot` | `boolean` | Boot the zone on system boot | `true` |
| `:bhyve` | `table` | See zone/bhyve |  |
| `:boot-after-install` | `string` | Boot the zone once it is installed | `true` |
| `:bootstrap-from` | `table` | Copy gurp into the zone, and apply the given file, relative to zone root |  |
| `:bootstrap` | `table` | See zone/bootstrap |  |
| `:capped-memory` | `struct` | Set memory cap. Keys must be :physical and :swap, values are strings like '4G' |  |
| `:clone-from` | `string` | Instead of installing, clone from the given zone, which must exist and be halted |  |
| `:copy-in` | `struct` | Copy files into the zone. Key (keyword) is src, val is dest, relative to zone root. Unqualified src is assumed to be in ../files/ |  |
| `:datasets` | `tuple` | ZFS datasets (as strings) to be delegated to zone |  |
| `:dns` | `struct` | DNS info. :domain is a string; :nameservers a tuple of strings |  |
| `:exec-in` | `tuple` | Runs the given commands (:string) in the zone after booting |  |
| `:final-state` | `string` | Put the zone in the given state. Also accepts 'reboot' |  |
| `:fs` | `array` | See zone/fs |  |
| `:hostid` | `string` | Force this hostid for the zone |  |
| `:ip-type` | `string` | IP type: exclusive or shared |  |
| `:limitpriv` | `tuple` | List of privileges to add to zone |  |
| `:lx-image` | `string` | Install zone using this image. See docs for pattern rules |  |
| `:net` | `array` | See zone/network |  |
| `:pool` | `string` | Resource pool to which zone should belong |  |
| `:rctl` | `array` | See zone/rctl |  |
| `:recreate` | `number` | 1-in-n chance the zone will be destroyed and recreated | `0` |
| `:zonepath` | `string` | Path to zone root |  |

## zone/remove

```janet
(zone/remove "defunct-zone")
```

### Mandatory Properties

None

### Optional Properties

None

