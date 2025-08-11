(use judge)
(use ../lib/gurp)

(deftest "smf-resources"
  (set *collector* (new-collector))

  (smf/ensure "telegraf"
              :description "Run Telegraf agent"
              :fmri "sysdef/telegraf"
              (smf-method "start"
                          :exec "/opt/site/lib/smf/method/telegraf.sh"
                          :user "telegraf"
                          :group "daemon"
                          :privileges ["basic" "file_dac_search" "sys_admin"
                                       "proc_owner" "proc_zone"])
              (smf-method "refresh" :exec ":kill -THAW"))

  (test *collector*
        @{:ensure @{:smf @[{:_id "/NO-ROLE/smf/telegraf"
                            :default-enabled true
                            :description "Run Telegraf agent"
                            :fmri "sysdef/telegraf"
                            :name "telegraf"
                            :refresh-method @{:exec ":kill -THAW" :timeout 60}
                            :single-instance true
                            :start-method @{:context {:group "daemon"
                                                      :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"
                                                      :user "telegraf"}
                                            :exec "/opt/site/lib/smf/method/telegraf.sh"
                                            :timeout 60}
                            :stop-method {:exec ":kill" :timeout 10}}]}
          :remove @{}}))

(deftest "smf-error"
  (test-error
    (smf/ensure "telegraf"
                :description "Run Telegraf agent"
                :fmri "sysdef/telegraf"
                (smf-method "start" :exec "/opt/site/lib/smf/method/telegraf.sh")
                (smf-method "refresh" :exec ":kill -THAW")
                (smf-method "gibbus" :exec "gibbus"))
    "action must be one of start, stop, refresh, reload")

  (test-error
    (smf/ensure "telegraf"
                (smf-method "start"
                            :exec "/opt/site/lib/smf/method/telegraf.sh"
                            :user "telegraf"
                            :group "daemon"))
    "Failed to validate user input for smf 'telegraf' : smf missing required key(s): description, fmri"))

(deftest smf-method
  (test
    (smf-method "start"
                :exec "/opt/site/lib/smf/method/telegraf.sh"
                :user "telegraf"
                :group "daemon"
                :privileges ["basic" "file_dac_search" "sys_admin" "proc_owner" "proc_zone"])
    {:start-method @{:context {:group "daemon"
                               :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"
                               :user "telegraf"}
                     :exec "/opt/site/lib/smf/method/telegraf.sh"
                     :timeout 60}}))
