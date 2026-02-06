## doers

`gurp doers` lists all the doers built into this version of Gurp.

```
Usage: gurp doers [OPTIONS]

Options:
  -C, --no-colour  Do not use any ANSI colouring
  -h, --help       Print help
```

For more information on any doer, use [the `describe` command](./describe.md).

```
$ gurp doers -C
               apk  Install and uninstall APK packages. Only valid in
                    an Alpine LX zone.
            bridge  Create and modify ethernet bridges.
              cron  Manage cron jobs. Crontab entries are prefixed with
                    a machine-generated string.
         directory  Create and remove directories. Parents are created
                    like mkdir -p, but with the owner/group/mode of the
                    gurp process. Removal always removes directory
                    contents.
         etherstub  Create and destroy etherstubs.
         file-line  Ensure lines do or do not exist in the given file.
              file  Create files from multiple sources, or remove them.
               gem  Install and uninstall Ruby gems.
             group  Create and destroy Unix groups.
        ip-address  Manages IP addresses via ipadm.
      ip-interface  Create and destroy IP interfaces, with optional
                    properties. Properties are supplied with
                    'ip-interface-protocol'.
     ip-properties  Sets global IP properties, via 'ipadm set-prop'.
             ipnat  Set or remove NAT rules.
              misc  A collection of things too small to deserve their
                    own doer.
      network-flow  Manage network flows via flowadm.
               pkg  Install and uninstall pkg(5) packages.
             pkgin  Install and uninstall pkgin packages. Only valid in
                    a pkgsrc zone.
         publisher  Add and remove pkg(5) publisher origins.
             route  Manage routes. Note that default routes for zones
                    should be handled by the zone's :defrouter
                    property.
               smf  Create and install a manifest for an SMF service.
               svc  Manage the state of an existing SMF service.
           svcprop  Manage properties of an existing SMF service.
           symlink  Create and remove symbolic links.
              user  Manage Unix users
              vlan  Manage VLAN objects
              vnic  Manage VNIC objects
               zfs  Create, destroy, and modify properties of ZFS
                    filesystems.
              zone  Create and destroy zones. Existing zones cannot be
                    modified.
```

By default the doer names are highlighted in bold. If you specify `-C`, or run
direct output in any way, the output is plain text.
