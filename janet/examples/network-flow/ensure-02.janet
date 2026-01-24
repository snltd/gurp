(network-flow/ensure "ssh-flow"
                     :link "vnic0"
                     :protocol "tcp"
                     :local-port 22
                     :maxbw "1M")
