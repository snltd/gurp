(import "../globals")

(role basenode
      (section nfs
               (misc/ensure :nfs-domain "lan.id264.net"))
      (section dirs
               (directory/ensure "/export" :group "sys")
               (directory/ensure "/export/home"))

      (section site
               (directory/ensure globals/site-dir)
               (directory/ensure globals/site-bin)
               (directory/ensure globals/site-etc)
               (directory/ensure globals/site-smf-method)
               (directory/ensure globals/site-smf-manifest))

      (section packages
               (pkg/ensure "library/readline")
               (pkg/ensure "shell/zsh")
               (pkg/ensure "ooce/library/yaml")
               (pkg/ensure "ooce/runtime/ruby-33")
               (pkg/ensure "ooce/text/ripgrep")
               (pkg/ensure "ooce/util/fd"))

      (section sudo
               (file/ensure "/etc/sudoers.d/sudo_group"
                            :mode "0400"
                            :content "%sysadmin ALL=(ALL:ALL) ALL"))

      (section users
               (user/ensure "rob"
                            :uid 264
                            :gecos "Rob Fisher"
                            :home-dir "/home/rob"
                            :password-hash "gwjggijwo"
                            :shell "/bin/zsh"
                            :group "sysadmin"))

      (section cron
               (file/ensure "/etc/default/cron"
                            :label "crondef"
                            :group "sys"
                            :content "CRONLOG=YES\nPATH=/bin:/sbin:/usr/sbin:/opt/oo/bin:/opt/ooce/sbin")
               (directory/ensure globals/cron-log-dir
                                 :mode "0755"
                                 :group "daemon")
               (svc/ensure "cron"
                           :restarted-by [(this "file" "crondef")]))

      (section good-sense
               (file-line/ensure "/etc/profile"
                                 :line "set -o vi")
               (file-line/ensure "/etc/profile"
                                 :line "PATH=${PATH}:/opt/ooce/bin")))
