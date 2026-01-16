(import ../globals)

(def executable (pathcat globals/site-bin "cron_monitor_dtrace"))
(def method-path (pathcat globals/site-smf-method "cron_monitor_dtrace"))

(role cron-monitor
      (pkg/ensure "network/netcat")

      (user/ensure "cronmon"
                   :uid 107
                   :primary-group "daemon"
                   :home-dir "/var/tmp"
                   :shell "/bin/false"
                   :gecos "cron_monitor pseudo-user")

      (file/ensure executable
                   :label "cron_monitor"
                   :mode "0755"
                   :from "cron-monitor/cron_monitor_dtrace")

      (file/ensure method-path
                   :mode "0755"
                   :from "cron-monitor/cron_monitor_dtrace_method")

      (svc/ensure "sysdef/cron_monitor"
                  :restarted-by (this :file :cron_monitor)
                  :state "online")

      (smf/ensure "cron_monitor"
                  :fmri "sysdef/cron_monitor"
                  :description "DTrace cron monitor"
                  (smf/method "start"
                              :exec method-path
                              :timeout 10
                              :user "cronmon"
                              :group "daemon"
                              :privileges ["basic"
                                           "!file_link_any"
                                           "dtrace_kernel"
                                           "dtrace_proc"
                                           "dtrace_user"])
                  :property-groups {:restarter "framework"}
                  :properties {:contract "fixed"
                               :max_restarts 10
                               :delay 10}))
