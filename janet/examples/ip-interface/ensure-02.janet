(ip-interface/ensure "example1"
                     :label "example-interface"
                     (ip-interface/protocol "ipv6"
                                            :mtu 1500
                                            :forwarding false)
                     (ip-interface/protocol "ipv4"
                                            :mtu 1500
                                            :forwarding true))
