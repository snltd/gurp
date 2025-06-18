(def default-protos
  {:file {:owner "root"
          :mode "0644"
          :group "root"}
   :svc {:state "online"
         :restarted-by []
         :reloaded-by []}
   :smf {:single-instance true
         :stop-method {:exec ":kill"
                       :timeout 10}
         :default-enabled true}
   :cron {:hour "*"
          :minute "*"
          :day-of-month "*"
          :day-of-week "*"
          :month-of-year "*"
          :user "root"}
   :user {:shell "/bin/zsh"
          :primary-group "staff"}
   :directory {:owner "root"
               :mode "0755"
               :group "root"}})
