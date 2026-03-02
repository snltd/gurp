(network-flow/ensure "ssh-flow"
                     :link "vnic1"
                     :protocol "tcp"
                     :local-port 22
                     :maxbw "1200K")
