(zfs/ensure "rpool/example/filesystem"
            :label "zfs-example-1"
            :properties {:compression "gzip-9"
                         :mountpoint "/example/mountpoint"
                         :dedup true
                         :devices false})
