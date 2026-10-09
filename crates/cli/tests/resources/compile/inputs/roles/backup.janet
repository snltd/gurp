(import ../globals)
(use ../helpers)

(def backup-root "/export/backup")
(def zpool "rpool/test-zone-dataset")

(role backup
      (zfs/ensure (zfscat zpool "backup"))

      (loop [client :in globals/backup-clients]
        (zfs/ensure (zfscat zpool client)
                    :properties {:mountpoint (pathcat backup-root client)
                                 :compression "gzip-9"
                                 :devices "off"
                                 :setuid "off"
                                 :exec "off"})

        (directory/ensure (pathcat backup-root client)
                          :owner "backup"
                          :group (this "user" "backup" "primary-group")
                          :mode "0700"))

      (section kronos
               (file/ensure (pathcat globals/site-bin "backup_kronos")
                            :from "backup/backup_kronos"
                            :mode "0755")

               (file/ensure (pathcat globals/site-bin "backup_kronos_metrics")
                            :from "backup/backup_kronos_metrics"
                            :mode "0755")

               (cron/ensure "kronos-backup-script"
                            :minute 26
                            :user "backup"
                            :command (site-cron "backup_kronos"))

               (cron/ensure "Report Kronos backup files to Wavefront"
                            :command (site-cron "backup_kronos_metrics")
                            :minute 25
                            :user "backup"))

      (section user
               (user/ensure "backup"
                            :gecos "Backup user"
                            :primary-group "daemon"
                            :shell "/bin/ksh"
                            :home-dir "/export/home/backup"
                            :password-hash "NP"
                            :uid 200)

               (zfs/ensure (zfscat zpool "user")
                           :properties {:mountpoint "/export/home/backup"})

               (directory/ensure (pathcat (this "user" "backup" "home-dir") ".ssh")
                                 :owner "backup"
                                 :group (this "user" "backup" "primary-group")
                                 :mode "0700")))
