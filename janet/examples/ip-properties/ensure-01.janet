(ip-properties/ensure "general"
                      :properties {:ipv6 {:hoplimit 123
                                          :hostmodel "weak"}
                                   :ipv4 {:hostmodel "weak"}
                                   :icmp {:max_buf 1234567}})
