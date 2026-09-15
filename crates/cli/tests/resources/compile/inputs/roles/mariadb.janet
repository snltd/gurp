(import ../globals)
(import ../helpers)

(def major-ver 11)
(def minor-ver 4)
(def svc-name (string/format "svc:/ooce/database/mariadb%d%d" major-ver minor-ver))
(def data-dir "/data")
(def backup-dir "/backup")
(def backup-user "db-backup")
(def db-user "mysql")
(def db-group "mysql")

(role mariadb
      (def mariadb-log-dir "/var/log/mariadb")

      (def my-config
        {:mysqld
         {:datadir data-dir
          :explicit_defaults_for_timestamp true
          :secure_file_priv ""
          :server-id 1
          :innodb_buffer_pool_size "128M"
          :log_bin 1
          :max_binlog_size "100M"
          :expire_logs_days 10
          :join_buffer_size "128M"
          :sort_buffer_size "2M"
          :read_rnd_buffer_size "2M"
          :slow_query_log 1
          :slow_query_log_file (pathcat mariadb-log-dir "slow_query.log")
          :long_query_time 0.25
          :log_queries_not_using_indexes 1
          :log_error (pathcat mariadb-log-dir "error.log")
          :sql_mode "STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,NO_ZERO_DATE,ERROR_FOR_DIVISION_BY_ZERO,NO_ENGINE_SUBSTITUTION"}})

      (def zfs-pool globals/fast-pool)

      (directory/ensure data-dir
                        :owner db-user
                        :group db-group
                        :mode "0700")

      (directory/ensure mariadb-log-dir
                        :owner db-user
                        :group db-group
                        :mode "2750")

      (file/ensure (string/format "/etc/opt/ooce/mariadb-%d.%d/my.cnf"
                                  major-ver minor-ver)
                   :label "my.cnf"
                   :from-struct my-config
                   :to-format "ini")

      (pkg/ensure (string/format "ooce/database/mariadb-%d%d" major-ver minor-ver))

      (zfs/ensure (zfscat zfs-pool "data")
                  :properties {:mountpoint data-dir})

      (svcprop/ensure (string svc-name ":default")
                      :property-groups {:application "application"}
                      :properties {:application/datadir data-dir})

      (svc/ensure svc-name
                  :state "online")

      (section "backup"
               (zfs/ensure (zfscat zfs-pool "backup")
                           :properties {:mountpoint backup-dir})

               (directory/ensure backup-dir
                                 :owner backup-user
                                 :mode "0700")

               (directory/ensure (pathcat backup-dir "archive")
                                 :owner backup-user
                                 :mode "0700")

               (directory/ensure (pathcat backup-dir "latest")
                                 :owner backup-user
                                 :mode "0700")

               (file/ensure (pathcat globals/site-bin "hot_backup")
                            :from "mariadb/mariadb-hot-backup.sh"
                            :mode "0755")

               (user/ensure backup-user
                            :gecos "mariadb backup user"
                            :primary-group "daemon"
                            :shell "/bin/ksh"
                            :home-dir "/export/home/backup"
                            :uid 700
                            :password-hash "NP")

               (cron/ensure "database-hot-backup"
                            :hour 15
                            :minute 58
                            :user backup-user
                            :command (helpers/site-cron "hot_backup"))))
