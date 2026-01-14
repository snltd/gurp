(use judge)
(import ../../../src/doers/smf)

(deftest method
  (test
    (smf/method "start"
                :exec "/opt/site/lib/smf/method/telegraf.sh"
                :user "telegraf"
                :group "daemon"
                :privileges ["basic" "file_dac_search" "sys_admin"
                             "proc_owner" "proc_zone"])
    {:start-method @{:context {:group "daemon"
                               :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"
                               :user "telegraf"}
                     :exec "/opt/site/lib/smf/method/telegraf.sh"
                     :timeout 60}}))

