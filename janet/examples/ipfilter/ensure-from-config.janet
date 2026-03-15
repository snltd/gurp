(ipfilter/ensure "rules-from-config"
                 :priority 0
                 :always-reload true
                 :content "block in log all\nblock out all")
