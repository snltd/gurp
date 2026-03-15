# zone

Create and destroy zones. Existing zones cannot be modified.

## Resource Name

Zone name (`:string`)

## zone/ensure

```janet
(zone/ensure "bhyve-zone"
             :brand "bhyve"
             :autoboot false
             (zone/network "bhyve0"
                           :allowed-address "192.168.1.102/24"
                           :global-nic "auto")
             (zone/bhyve
               :ram "4G"
               :vcpus 4
               :image-path "/var/tmp/noble-server-cloudimg-amd64.img.raw"
               :boot-volume "tank/bhyve/test"
               :cloudinit-struct {:network {:version 2}})
             :dns {:domain "lan.id264.net"
                   :nameservers ["192.168.1.53"
                                 "192.168.1.1"]})
```

```janet
(zone/ensure "native-zone"
             :brand "lipkg"
             :clone-from "gold-zone"
             (zone/fs "/home"
                      :special "/export/home")
             (zone/network "test_net0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.101/24"
                           :defrouter "192.168.1.1")
             (zone/bootstrap
               :server "gurp.localnet"
               :hostname "native-zone"))
```

```janet
(zone/ensure "lx-zone"
             :brand "lx"
             :lx-image "alpine"
             :final-state "reboot"
             (zone/network "znet0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.103/24"
                           :defrouter "192.168.1.1")
             (zone/attr "kernel-ver" :value "4.4")
             :exec-in ["/bin/exec1" "/bin/exec2"]
             :copy-in {"lx-test/f1" "/etc/file1"
                       "lx-test/f2" "/bin/exec2"})
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
(zone/remove "unwanted-zone")
```

### Mandatory Properties

None

### Optional Properties

None

## Notes

- `kvm` and `illumos` zones are not supported.
- You cannot modify an existing zone in-place.
- `recreate` must be an integer, and it is the n:1 odds of a zone being destroyed and recreated. So, `0` means "never recreate this zone", and `1` means "recreate this zone on every run". You can set the number as high as you like, so if you run Gurp every 15 minutes and want your zone rebuilt from scratch about once a week, you'd use `:recreate 672`.
# zone/attr

Set attributes on a zone being created by the zone doer.

## Name

Attribute name (`:string`)

```janet
(zone/attr "kernel-ver"
  :value "4.4")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:name` | `string` | Attribute name. Derived from resource name |  |
| `:value` | `string boolean number` | Attribute value |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:type` | `string` | The type of the value. Gurp will take a pretty good guess though |  |


# zone/bhyve

Describe a bhyve zone inside a zone resource.

## Name

This helpers does not accept a name

```janet
(use ../../src/dsl)

(zone/bhyve
  :vcpus 4
  :ram "8G"
  :image-url "https://cloud-images.ubuntu.com/noble/current/noble-server-cloudimg-amd64.img"
  :image-format "qcow2"
  :boot-volume "tank/byhve/example-boot"
  :cloudinit-files [(config-file "cloud-init/user-data")]
  :cloudinit-struct
  {:meta-data (cloudinit-meta-data "example-zone")

   :network-config
   {:network {:version 2
              :ethernets
              {:enp0s6 {:addresses ["10.10.0.2"]
                        :mtu 1500
                        :nameservers {:search ["localnet"]
                                      :addresses ["10.10.0.53" "1.1.1.1"]}
                        :routes [{:to "0.0.0.0/0"
                                  :via "10.10.0.1"}]}}}}})
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:boot-volume` | `string` | ZFS boot volume |  |
| `:ram` | `string` | Amount of RAM to allocate: e.g. '3G' |  |
| `:vcpus` | `number` | Number of VCPUs to allocate |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:cloudinit-files` | `tuple` | Copy the given files into the Cloudinit image |  |
| `:cloudinit-struct` | `struct` | Generate a Cloudinit file from the given struct. Top level keys map to files, e.g. 'user-data' |  |
| `:image-format` | `string` | Specify the format of the image pointed to by :image-url |  |
| `:image-path` | `string` | Path to install image - must be raw format |  |
| `:image-url` | `string` | URL of remote install image |  |
| `:wait-for-boot` | `boolean` | Wait for boot, or detach immediately | `true` |

## Notes

- The `:boot-volume` must already exist. It is not created by this doer.
- You must supply exactly one of `:image-url` and `:image-path`

# zone/bootstrap

Tells gurp how to bootstrap a newly created zone.

## Name

This helpers does not accept a name

```janet
(zone/bootstrap
  :file "/path/inside/zone")
```

```janet
(zone/bootstrap
  :server "gurp.localnet"
  :hostname "networked-zone")
```

### Mandatory Properties

None

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:file` | `string` | fully qualified path of file in zone which will be used to bootstrap |  |
| `:hostname` | `string` | hostname of client being bootstrapped |  |
| `:server` | `string` | hostname/IP address of server to install from |  |

## Notes

- You must supply exactly one of `:file` and `:server`.

# zone/fs

Define a filesystem mapping when creating a zone.

## Name

The mountpoint inside the zone (`:string`)

```janet
(zone/fs "/dir/in/zone"
         :special "/dir/in/global"
         :type "lofs"
         :options ["ro"])
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:dir` | `string` | Mountpoint in zone. This is the name of the resource, and is not specified with a key |  |
| `:special` | `string` | The directory in the global zone |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:options` | `tuple` | Options with which to mount fs inside zone |  |
| `:type` | `string` | The type of fs mount | `"lofs"` |


# zone/network

Describe network configuration of a zone resource.

## Name

Zone VNIC, which may already exist (`:string`)

```janet
(zone/network "znet0"
              :global-nic "e1000g1"
              :physical "znic0"
              :allowed-address "192.168.1.103/24"
              :defrouter "192.168.1.1")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:physical` | `string` | Zone VNIC. This is the name of the resource, and is not specified with a key |  |

### Optional Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:allowed-address` | `string` | IP address, with /netmask |  |
| `:defrouter` | `string` | IP address of default router |  |
| `:global-nic` | `string` | Physical NIC on which to create zone VNIC | `"auto"` |


# zone/rctl

Define a resource control when creating a zone.

## Name

RCTL name (`:string`)

```janet
(zone/rctl "example"
           :priv "zone.cpu-cap"
           :limit 300
           :action "none")
```

### Mandatory Properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:action` | `string` | rctl action | `"deny"` |
| `:limit` | `number` | rctl limit value |  |
| `:name` | `string` | private field managed by Gurp |  |
| `:priv` | `string` | rctl privilege | `"privileged"` |

### Optional Properties

None

