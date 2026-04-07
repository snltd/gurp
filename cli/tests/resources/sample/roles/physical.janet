(def resolv-conf (indoc `
  domain lan.id264.net
  nameserver 192.168.1.53
  nameserver 192.168.1.1`))

(def ntp-conf (indoc `
  statsdir /var/log/ntpstats

  restrict 127.0.0.1

  server 0.pool.ntp.org iburst
  server 1.pool.ntp.org iburst
  server 2.pool.ntp.org iburst
  server 3.pool.ntp.org iburst

  driftfile /var/lib/ntp/drift`))

(role physical
      (zfs/ensure "rpool/zones/gurp/test"
                  :properties {:compression "on"
                               :devices "on"})
      (section "dns"
               (file/ensure "/etc/resolv.conf" :content resolv-conf))

      (section "ntp"
               (pkg/ensure "service/network/ntp")
               (directory/ensure "/var/lib/ntp" :group "daemon")
               (file/ensure "/etc/ntp.conf" :content ntp-conf :label "ntp-conf"))
      # (svc/ensure "svc:/network/ntp:default" :restarted-by (this "file" "ntp-conf")))

      (section "telemetry"
               (smf/ensure "telegraf"
                           :description "Run Telegraf agent"
                           :fmri "sysdef/telegraf"
                           :start-method {:exec "/bin/sleep 1200"
                                          :timeout 60
                                          :context {:user "telegraf"
                                                    :group "daemon"
                                                    :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"}}
                           :stop-method {:exec ":kill"
                                         :timeout 10}
                           :refresh-method {:exec ":kill -THAW"
                                            :timeout 60}))

      (section "packages"
               (pkg/ensure "network/rsync")
               (pkg/ensure "shell/zsh")
               (pkg/ensure "system/zones/brand/bhyve")
               (pkg/ensure "system/zones/brand/illumos")
               (pkg/ensure "system/zones/brand/ipkg")
               (pkg/ensure "system/zones/brand/lipkg")
               (pkg/ensure "system/zones/brand/lx/platform")
               (pkg/ensure "system/zones/brand/pkgsrc")
               (pkg/ensure "system/zones/brand/sparse")))
