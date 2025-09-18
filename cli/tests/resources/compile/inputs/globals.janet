(def site-dir "/opt/site")
(def site-bin (string site-dir "/bin"))
(def site-etc (string site-dir "/etc"))
(def site-smf-manifest (string site-dir "/lib/smf/manifest"))
(def site-smf-method (string site-dir "/lib/smf/method"))
(def cron-log-dir "/var/log/cron_jobs")
(def gem-dir "/opt/ooce/bin")

(def big-pool "big")
(def fast-pool "fast")

(def local-domain "lan.id264.net")
(def netmask "24")

(def backup-clients ["kronos"])

(def pkg-url (string "http://pkg." local-domain "/"))

(def hostname->ip
  {:router "192.168.1.1"
   :serv "192.168.1.5"
   :kronos "192.168.1.6"
   :slim "192.168.1.9"
   :pch "192.168.1.10"
   :lipkg-blue "192.168.1.19"
   :lipkg-green "192.168.1.20"
   :serv-ws "192.168.1.21"
   :serv-proxy "192.168.1.22"
   :serv-build "192.168.1.23"
   :serv-pkg "192.168.1.24"
   :serv-media "192.168.1.25"
   :serv-grafana "192.168.1.26"
   :serv-mariadb "192.168.1.27"
   :serv-fs "192.168.1.28"
   :serv-backup "192.168.1.29"
   :serv-metrics "192.168.1.30"
   :serv-cron "192.168.1.31"
   :serv-records "192.168.1.32"
   :serv-dns "192.168.1.53"
   :kate-docker "192.168.1.79"
   :kate-k8s-00 "192.168.1.80"
   :kate-k8s-01 "192.168.1.81"
   :kate-k8s-02 "192.168.1.82"
   :st2000-lom "192.168.1.130"
   :st2000 "192.168.1.150"})

(def cname->hostname
  {:build "serv-build"
   :dns "serv-dns"
   :grafana "serv-proxy"
   :mariadb "serv-mariadb"
   :metrics "serv-metrics"
   :mysql "serv-mariadb"
   :pkg "serv-pkg"
   :records "serv-proxy"
   :switch "sw-16p"
   :ws "serv-ws"})

(def zone-dns
  {:domain local-domain
   :nameservers [(hostname->ip :serv-dns)
                 (hostname->ip :router)]})
