(use judge)
(use ../lib/gurp)

(deftest "ensure-smf-manifest"
  (test 
        (smf/ensure "telegraf"
                    :description "Run Telegraf agent"
                    :fmri "sysdef/telegraf"
                    :start-method {
                      :exec "/opt/site/lib/smf/method/telegraf.sh"
                      :context {
                        :user "telegraf"
                        :group "daemon"
                        :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"
                      }
                    }
                    :refresh-method {
                      :exec ":kill -THAW"
                    }

)
    {:smf {:_id "/NO-ROLE/smf/telegraf"
           :action :ensure
           :default-enabled true
           :description "Run Telegraf agent"
           :fmri "sysdef/telegraf"
           :name "telegraf"
           :refresh-method {:exec ":kill -THAW"}
           :single-instance true
           :start-method {:context {:group "daemon"
                                    :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"
                                    :user "telegraf"}
                          :exec "/opt/site/lib/smf/method/telegraf.sh"
                          :timeout 60}
           :stop-method {:exec ":kill" :timeout 10}
           :svc-name "telegraf"}}))
