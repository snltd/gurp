(zfs/ensure (zfscat "tank" "export" "test")
            :label "test-zfs"
            :properties {:compression "gzip9"
                         :devices "off"})
