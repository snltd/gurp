(ip-properties/ensure "general"
                      :ipv4 {:forwarding true}
                      :ipv6 {:hoplimit 250}
                      :icmp {:max_buf 262000}
                      :tcp {:sack "passive"}
                      :udp {:extra_priv_ports "2050,4040"}
                      :sctp {:max_buf 1048000})
