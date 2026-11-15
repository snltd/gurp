(file/ensure "/example/file/from-url"
             :from-url "https://raw.githubusercontent.com/snltd/gurp/refs/heads/main/LICENSE.txt"
             :label "remote-file"
             (file/checksum
               :algorithm "sha256"
               :literal "7bde5db38472e2c187ad2e5e10c25a4aecb59e8304c26b9894d11c4b99196ea7"))
