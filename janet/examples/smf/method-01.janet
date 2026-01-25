(smf/method "start"
            :exec "/opt/site/lib/smf/method/example.sh"
            :user "example"
            :group "daemon"
            :privileges ["basic"
                         "file_dac_search"
                         "sys_admin"
                         "proc_owner"
                         "proc_zone"])
