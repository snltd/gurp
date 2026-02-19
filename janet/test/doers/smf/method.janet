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
                     :timeout 60}})

  (test-error
    (smf/method "explode" :exec "boom!")
    "In smf/method explode: smf/method name must be one of \"start\", \"stop\", \"refresh\", \"reload\"")

  (test-error
    (smf/method "start" :exec "/bin/prog" :role "boss")
    "In smf/method start: unexpected property :role. Valid properties are :exec, :timeout, :user, :group, :environment, :label, :privileges")

  (test-error
    (smf/method "start" :thing "whatever")
    "In smf/method start: did not find mandatory property :exec. Mandatory properties are :exec, :timeout"))
