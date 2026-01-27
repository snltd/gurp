(file/ensure "/file/from/arbitrary/server"
             :owner "gibbus"
             :mode "0640"
             :with-checksum "0123456789abcdef"
             :from-url "https://example.com/files/config")
