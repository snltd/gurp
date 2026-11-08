(directory/ensure "/example/dir_4"
                  :create-from "http://example.com/files/archive.tar"
                  (file/checksum
                    :algorithm "sha512"
                    :suffix "sha512"))
