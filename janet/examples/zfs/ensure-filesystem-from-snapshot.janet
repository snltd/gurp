(zfs/ensure "rpool/example"
            :create-from "http://example.com/zfs/example.zfs"
            :properties {:atime false
                         :mtime false
                         :exec false
                         :compression "lz4"}
            (file/checksum
              :algorithm "sha256"
              :literal "--checksum-goes-here--"))
