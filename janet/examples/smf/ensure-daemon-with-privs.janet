(smf/ensure "example"
            :description "Run example program"
            :fmri "snltd/example"
            :duration "child"
            (smf/dependency "dependency1"
                            :fmri "svc:/milestone/name-services:default")
            (smf/dependency "dependency2"
                            :grouping "optional_all"
                            :restart-on "error"
                            :fmri "svc:/system/pkgserv:default")
            (smf/method "start"
                        :exec "/app/method.sh"
                        :user "appuser"
                        :group "daemon"
                        :privileges ["basic"
                                     "!file_dac_search"])
            :property-groups {:application "application"
                              :other_group "framework"}
            :properties {:application/port 8080
                         :application/ssl true
                         :other_group/other_prop "abc123"})
