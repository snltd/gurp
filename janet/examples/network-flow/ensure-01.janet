  (network-flow/ensure "tls-throttle"
                       :link "vnic1"
                       :protocol "tcp"
                       :remote-port 443
                       :maxbw "10M")
