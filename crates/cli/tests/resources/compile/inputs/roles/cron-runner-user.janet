(role cron-runner-user
      (user/ensure "cron"
                   :gecos "cron job runner"
                   :primary-group "daemon"
                   :shell "/bin/ksh"
                   :home-dir "/export/home/cron"
                   :password-hash "NP"
                   :uid 105)

      (directory/ensure (this "user" "cron" "home-dir")
                        :owner "cron"))
