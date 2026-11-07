(file/ensure "/example/file/from-url"
             :from-url "https://raw.githubusercontent.com/snltd/gurp/refs/heads/main/LICENSE.txt"
             :label "remote-file"
             (file/checksum
               :algorithm "sha256"
               :literal "561a47aa1d1bfc3a95ce45345639f9ce2d9ad332b05cfe5da74ad77f2842ee16"))
