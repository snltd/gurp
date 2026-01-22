(smf/ensure "telegraf"
            :description "Run Telegraf agent"
            :fmri "sysdef/telegraf"
            (smf/dependency "svc1"
                            :fmri "svc://example/service1:default")
            (smf/dependency "svc2"
                            :grouping "optional-all"
                            :restart-on "error"
                            :fmri "svc://example/service2:default")
            (smf/method "start"
                        :exec "/opt/site/lib/smf/method/telegraf.sh"
                        :user "telegraf"
                        :group "daemon"
                        :privileges ["basic" "file_dac_search" "sys_admin"
                                     "proc_owner" "proc_zone"])
            :property-groups {:application "application"}
            :properties {:application/datadir "/data"})
