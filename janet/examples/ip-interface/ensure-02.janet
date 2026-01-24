(ip-interface/ensure "example1"
                     :label "example-interface"
                     :ipv6 {:mtu 1500
                            :forwarding false}
                     :ipv4 {:mtu 1500
                            :forwarding true})
