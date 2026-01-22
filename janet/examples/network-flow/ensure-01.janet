  (network-flow/ensure "tls_shaper"
                       :link "vnic1"
                       :protocol "tcp"
                       :remote-ip "203.0.113.4"
                       :remote-port 443
                       :maxbw "10M")
