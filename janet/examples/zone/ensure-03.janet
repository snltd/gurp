(zone/ensure "test-lx-zone"
             (zone/network "test_net0"
                           :global-nic "auto"
                           :allowed-address "192.168.1.33/24"
                           :defrouter "192.168.1.1")
             (zone/attr "kernel-ver" :value "4.4")
             :exec-in ["/bin/exec1" "/bin/exec2"]
             :copy-in {"lx-test/f1" "/etc/file1"
                       "lx-test/f2" "/bin/exec2"}
             :brand "lx")
