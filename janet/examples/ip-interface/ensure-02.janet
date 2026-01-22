(ip-interface/ensure "test-vnic1"
                     :label "merp"
                     (ip-interface/protocol "ipv6"
                                            :mtu 1500
                                            :forwarding false)
                     (ip-interface/protocol "ipv4"
                                            :mtu 1500
                                            :forwarding true))
