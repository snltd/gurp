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

    :smf
    {:single-instance true
     :stop-method {:exec ":kill" :timeout 10}
     :default-enabled true}

    :smf-method
    {:timeout 60}

    :svc
    {:state "online"
     :restarted-by []
     :reloaded-by []}

    :user
    {:shell "/bin/zsh"
     :primary-group "staff"}

    :zfs
    {:properties {:mountpoint: "none"}}

    :zone
    {:autoboot true
     :recreate 0
     :boot-after-install true}

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
