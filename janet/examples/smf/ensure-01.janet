(smf/ensure "example"
            :description "Run example program"
            :fmri "snltd/example"
            (smf/dependency "dependency1"
                            :fmri "svc://example/service1:default")
            (smf/dependency "dependency2"
                            :grouping "optional-all"
                            :restart-on "error"
                            :fmri "svc://example/service2:default")
            (smf/method "start"
                        :exec "/opt/site/lib/smf/method/example.sh"
                        :user "example"
                        :group "daemon"
                        :privileges ["basic"
                                     "file_dac_search"
                                     "sys_admin"
                                     "proc_owner"
                                     "proc_zone"])
            :property-groups {:application "application"}
            :properties {:application/datadir "/data"})
