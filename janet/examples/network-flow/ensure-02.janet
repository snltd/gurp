(network-flow/ensure "flow-ssh-test"
                     :link "vnic0"
                     :protocol "tcp"
                     :local-port 22
                     :maxbw "1M")
