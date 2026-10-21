# zone

Create and destroy zones. Existing zones cannot be modified.

## zone/ensure

### Resource Name

Zone name (`:string`)

### Mandatory properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:autoboot` | `boolean` | Boot the zone when the system boots. | `true` |
| `:boot-after-install` | `boolean` | Boot the zone once it is installed. | `true` |
| `:brand` | `string` | Zone brand. |  |

### Optional properties

|  key  |  type  |  description  |  default  |
|-------|--------|---------------|-----------|
| `:attr` | `array` | See zone/attr. |  |
| `:bhyve` | `table` | See zone/bhyve. |  |
| `:bootstrap-from` | `table` | Copy gurp into the zone, and apply the given file, relative to zone root. |  |
| `:bootstrap` | `table` | See zone/bootstrap. |  |
| `:capped-memory` | `struct` | Set memory cap. |  |
| `:clone-from` | `string` | Instead of installing, clone from the given zone, which must exist and be halted. |  |
| `:cloudinit` | `array` | See zone/cloudinit. |  |
| `:copy-in` | `struct` | Copy files into the zone. Key is source, value is dest, relative to zone root. If key is an array of strings, all files listed are copied to dest. Unqualified src is assumed to be in `../files`. Directories are copied recursively, and if the dest directory does not exist, it is created.. |  |
| `:datasets` | `tuple` | ZFS datasets (as strings) to be delegated to zone. |  |
| `:dns` | `struct` | DNS client configuration. |  |
| `:emu` | `table` | See zone/emu. |  |
| `:exec-in` | `tuple` | Runs the given commands (`:string`) in the zone after booting. |  |
| `:final-state` | `string` | Put the zone in the given state. Also accepts 'reboot'. |  |
| `:fs` | `array` | See zone/fs. |  |
| `:hostid` | `string` | Force this hostid for the zone. |  |
| `:image-checksum` | `struct table` | Define a checksum for a from-type resource. |  |
| `:image` | `string` | Install zone using this image. See docs for pattern rules. |  |
| `:ip-type` | `string` | IP type. Allowed values: "exclusive", "shared". |  |
| `:limitpriv` | `tuple` | List of privileges to add to zone. |  |
| `:network` | `array` | See zone/network. |  |
| `:pool` | `string` | Resource pool to which zone should belong. |  |
| `:rctl` | `array` | See zone/rctl. |  |
| `:recreate` | `number` | 1-in-n chance the zone will be destroyed and recreated. |  |
| `:zonepath` | `string` | Path to zone root. Defaults to `/zones/<zone-name>`. |  |

### Examples

```janet
(zone/ensure "bhyve-zone"
             :brand "bhyve"
             :autoboot false
             :image "/var/tmp/noble-server-cloudimg-amd64.img.raw"

             (zone/network "bhyve0"
                           :allowed-address "10.10.0.2/24"
                           :global-nic "auto")
             (zone/bhyve
               :ram "4G"
               :vcpus 4
               :boot-volume "tank/bhyve/test")

             (zone/cloudinit "meta-data"
                             :from-struct (cloudinit-meta-data "example-zone"))
             (zone/cloudinit "network"
                             :from-struct {:network {:fancy "struct"}})
             (zone/cloudinit "users" :from "cloudwatch/users")
             (zone/cloudinit "packages" :from "cloudwatch/packages"))
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
             :image "alpine"
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

```janet
(zone/ensure "emu-example"
             :brand "emu"
             :autoboot true
             :recreate 1
             :image "/var/tmp/braich-151059.raw"

             (zone/network "emu0"
                           :allowed-address "192.168.1.102/24"
                           :global-nic "auto")

             (zone/attr "type" :value "generic")
             (zone/attr "diskif" :value "virtio-blk-device")
             (zone/attr "netif" :value "virtio-net-device")

             (zone/rctl "zone.cpu-shares"
                        :priv "privileged"
                        :limit 1
                        :action "none")

             (zone/emu
               :bios "https://downloads.omnios.org/media/braich/u-boot.bin"
               :arch "aarch64"
               :vcpus 4
               :cpu "cortex-a53"
               :qemu-args ["-machine virt" "-accel tcg,thread=multi"]
               :ram "2G"
               :image-format "raw"
               :boot-volume "tank/emu-example"))
```

## zone/remove

### Resource Name

Zone name (`:string`)

### Mandatory properties

None

### Optional properties

None

### Examples

```janet
(zone/remove "unwanted-zone")
```

## Notes

- `kvm` zones are not supported.
- You cannot modify an existing zone in-place.
- When building an illumos zone, you can use the `:image` key to specify a fully qualified path or URL to the install image. If you do not specify an image, Gurp will fetch the latest OmniOS non-global zone ZFS dataset. Images are cached in `/var/tmp`.
- `recreate` must be an integer, and it is the n:1 odds of a zone being destroyed and recreated. So, `0` means "never recreate this zone", and `1` means "recreate this zone on every run". You can set the number as high as you like, so if you run Gurp every 15 minutes and want your zone rebuilt from scratch about once a week, you'd use `:recreate 672`.
