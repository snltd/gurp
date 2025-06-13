(def default-protos
  {:file {:owner "root"
          :mode "0644"
          :group "root"}
   :svc {:state "online"
         :restarted-by []
         :reloaded-by []}
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
