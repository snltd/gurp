(zone/ensure "lx-zone"
             :brand "lx"
             :image "alpine"
             :final-state "reboot"
             (zone/network "znet0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.103/24"
                           :defrouter "192.168.1.1")
             (zone/attr "kernel-ver" :value "4.4")
             :exec-in ["/bin/exec1" "/bin/exec2"]
             :copy-in {"lx-test/f1" "/etc/file1"
                       "lx-test/f2" "/bin/exec2"})
