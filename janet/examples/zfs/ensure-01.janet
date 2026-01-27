(zfs/ensure "tank/example/filesystem"
            :label "zfs-example-1"
            :properties {:compression "gzip9"
                         :mountpoint "/example/mountpoint"
                         :dedup true
                         :devices false})
