(system-cert/ensure "from-url"
                    :from-url "https://cert-service/api"
                    (file/checksum
                      :algorithm "sha256"
                      :with-suffix ".sha256"))
