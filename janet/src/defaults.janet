(def default-protos
  {:ensure
   {:cron
    {:hour "*"
     :minute "*"
     :day-of-month "*"
     :day-of-week "*"
     :month-of-year "*"
     :user "root"}

    :directory
    {:owner "root"
     :mode "0755"
     :group "root"}

    :file
    {:owner "root"
     :mode "0644"
     :group "root"}

    :route
    {:force-gateway false}
    
    :svc

    :user
    {:shell "/bin/zsh"
     :primary-group "staff"}

    :vnic
    {:with-interface false}

    :zfs
    {:properties {:mountpoint: "none"}}

    :zone
    {:autoboot true
     :recreate 0
     :boot-after-install true}

    :zone-bhyve
    {:wait-for-boot true}

    :zone-fs
    {:type "lofs"}

    :zone-network
    {:global-nic "auto"}

    :zone-rctl
    {:priv "privileged"
     :action "deny"}}

   :remove
   {:file-line
    {:match "exact"
     :apply-to "all"}}})
