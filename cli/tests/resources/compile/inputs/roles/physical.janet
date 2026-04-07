(import ../helpers)
(import ../globals)

(def resolv.conf (indoc `
  domain {{ domain }}
  nameserver {{ dns-server }}
  nameserver {{ router }}`))

(def ntp.conf (indoc `
  statsdir /var/log/ntpstats

  restrict 127.0.0.1

  server 0.uk.pool.ntp.org iburst
  server 1.uk.pool.ntp.org iburst
  server 2.uk.pool.ntp.org iburst
  server 3.uk.pool.ntp.org iburst

  driftfile /var/lib/ntp/drift`))

(role physical
      (section "dns"
               (file/ensure "/etc/resolv.conf"
                            :content (template-out resolv.conf
                                                   {:domain globals/local-domain
                                                    :dns-server (helpers/ip-of :serv-dns)
                                                    :router (helpers/ip-of :router)})))

      (section "ntp"
               (pkg/ensure "service/network/ntpsec")
               (directory/ensure "/var/lib/ntp" :group "daemon")
               (file/ensure "/etc/ntp.conf"
                            :content ntp.conf
                            :label "ntp-conf")
               (svc/ensure "svc:/network/ntp:default"
                           :state "online"
                           :restarted-by [:/physical/file/ntp-conf]))

      (misc/ensure :scheduler "FSS")

      (cron/ensure "turn-off-at-night"
                   :minute 0
                   :hour 1
                   :command "/usr/sbin/poweroff")

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
