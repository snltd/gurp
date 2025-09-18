(import ../globals)
(import ../secrets)

(def zfs-mounter "/etc/init.d/zfs-mount")
(def data-dir "/var/lib/grafana")
(def grafana-init "/etc/init.d/grafana")

(indoc cron-job ```
         #!/bin/sh

         /var/tmp/gurp \
           apply \
           --metrics-to=metrics \
           /home/rob/work/my-gurp/zone-grafana.janet \
         >/var/log/gurp.log 2>&1
         ```)

(role grafana
      (def zfs-pool globals/fast-pool)

      (zfs/ensure zfs-pool)

      (zfs/ensure (zfscat zfs-pool "data")
                  :properties {:mountpoint data-dir})

      (file/ensure "/etc/periodic/15min/gurp"
                   :mode "0755"
                   :content cron-job)

      # An LX Zone can see delegated datasets, but it won't mount them unless
      # it's told to.
      (section mount-zfs-filesystems
               (indoc zfs-mount-script ```
                    #!/sbin/openrc-run

                    description="Mount delegated ZFS dataset"

                    depend()
                    {
                        # run as early as possible, only after local filesystems
                        need localmount
                        before *
                    }

                    start()
                    {
                        ebegin "Mounting ZFS dataset"
                        /native/usr/sbin/zfs mount -a
                        eend $?
                    }```)

               (file/ensure zfs-mounter
                            :content zfs-mount-script
                            :mode "0755")

               (symlink/ensure "/etc/runlevels/boot/zfs-mount"
                               :source zfs-mounter))

      (section configue-grafana
               (def grafana-config
                 {:paths {:data data-dir
                          :logs "/var/log/grafana"
                          :plugins (pathcat data-dir "plugins")
                          :provisioning "conf/provisioning"}
                  :server {:protocol "http"
                           :http_port 3000}
                  :database {:type "mysql"
                             :host "mysql"
                             :name "grafana"
                             :user "grafana"
                             :password secrets/grafana-mysql-password}
                  :log {:mode "file"
                        :level "info"}
                  :news {:news_feed_enabled false}
                  :metrics {:enabled true}})

               (apk/ensure "grafana")

               (file-line/remove grafana-init
                                 :match "contains"
                                 :pattern "need net")

               (symlink/ensure "/etc/runlevels/default/grafana"
                               :source grafana-init)

               (file-line/ensure "/etc/conf.d/grafana"
                                 :replace "127.0.0.1" :with "0.0.0.0")

               (file/ensure "/etc/grafana.ini"
                            :from-struct grafana-config
                            :to-format "ini")))
