(import globals)
(import helpers)

(def global-name "serv")
(def gold-zone "lipkg-blue")
(def router (helpers/ip-of "router"))
(def gurp-dir "/home/rob/work/my-gurp")

(defn recreate?
  "If GURP_RECREATE_ZONE is a zone name, recreate that zone. If it's all, do them all"
  [zone-name]
  (def env-val (os/getenv "GURP_RECREATE_ZONE"))
  (if (or (= env-val "ALL") (= env-val zone-name)) 1 200))

(defn network [zone-name]
  (def short-name (->> zone-name (string/split "-") (last)))
  (zone/network (string short-name "_net0")
                :allowed-address (helpers/ip-of zone-name :with-netmask true)
                :defrouter router))

(defn zone-dataset
  [zone-short-name]
  (zfscat globals/fast-pool "zone" zone-short-name))

(role zones
      (zone/ensure gold-zone
                   :brand "lipkg"
                   :autoboot false
                   :recreate 0
                   (zone/fs "/home" :special "/export/home")
                   (network gold-zone)
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-gold.janet"))
                   :final-state "installed"
                   :exec-in ["/usr/bin/pkg refresh"
                             "/usr/bin/pkg update pkg | cat "
                             "/usr/bin/pkg update | cat"
                             "/usr/sbin/shutdown -y -i5 -g0"])

      (zfs/ensure (zone-dataset "pkg")
                  :properties {:mountpoint "none"})

      (zone/ensure "serv-pkg"
                   :brand "lipkg"
                   :recreate (recreate? "serv-pkg")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   :datasets [(zone-dataset "pkg")]
                   (network "serv-pkg")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-pkg.janet")))

      (zone/ensure "serv-dns"
                   :brand "lipkg"
                   :recreate (recreate? "serv-dns")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   (network "serv-dns")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-dns.janet")))

      (zfs/ensure (zone-dataset "backup")
                  :properties {:mountpoint "none"})

      (zone/ensure "serv-backup"
                   :brand "lipkg"
                   :recreate (recreate? "serv-backup")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   :capped-memory {:physical "300m" :swap "300m"}
                   :datasets [(zone-dataset "backup")]
                   (network "serv-backup")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-backup.janet")))

      (zfs/ensure (zone-dataset "build")
                  :properties {:mountpoint "none"})

      (zone/ensure "serv-build"
                   :brand "lipkg"
                   :recreate (recreate? "serv-build")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   :datasets [(zone-dataset "build")]
                   (network "serv-build")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-build.janet")))

      (zone/ensure "serv-cron"
                   :brand "lipkg"
                   :recreate (recreate? "serv-cron")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   :capped-memory {:physical "300m" :swap "300m"}
                   (network "serv-cron")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-cron.janet")))

      (zfs/ensure (zfscat globals/big-pool "zone" "fs")
                  :properties {:mountpoint "none"})

      (zone/ensure "serv-fs"
                   :brand "lipkg"
                   :autoboot false
                   :recreate (recreate? "serv-dns")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   :datasets [(zfscat globals/big-pool "zone" "fs")]
                   (network "serv-fs")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-fs.janet")))

      (zfs/ensure (zone-dataset "mariadb")
                  :properties {:mountpoint "none"})

      (zone/ensure "serv-mariadb"
                   :brand "lipkg"
                   :recreate (recreate? "serv-mariadb")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   :datasets [(zone-dataset "mariadb")]
                   (network "serv-mariadb")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-mariadb.janet")))

      (zfs/ensure (zone-dataset "metrics")
                  :properties {:mountpoint "none"})

      (zone/ensure "serv-metrics"
                   :brand "lipkg"
                   :recreate (recreate? "serv-dns")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   :datasets [(zone-dataset "metrics")]
                   (network "serv-metrics")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-metrics.janet")))

      (zone/ensure "serv-media"
                   :brand "lipkg"
                   :recreate (recreate? "serv-media")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   (zone/fs "/storage/mp3"
                            :special "/export/mp3"
                            :options ["ro"])
                   (zone/fs "/storage/flac"
                            :special "/export/flac"
                            :options ["ro"])
                   (network "serv-media")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-media.janet")))

      (zfs/ensure (zone-dataset "grafana")
                  :properties {:mountpoint "none"})

      (zone/ensure "serv-grafana"
                   :brand "lx"
                   :recreate (recreate? "serv-grafana")
                   :image "alpine"
                   :final-state "reboot"
                   (zone/attr "kernel-version" :value "4.4")
                   (zone/fs "/home" :special "/export/home")
                   (network "serv-grafana")
                   :dns globals/zone-dns
                   :datasets [(zone-dataset "grafana")]
                   (zone/bootstrap :file (pathcat gurp-dir "zone-grafana.janet")))

      (zone/ensure "serv-ws"
                   :brand "lipkg"
                   :recreate (recreate? "serv-ws")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   (zone/fs "/storage" :special "/export")
                   (network "serv-ws")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-ws.janet")))

      (zone/ensure "serv-proxy"
                   :brand "lipkg"
                   :recreate (recreate? "serv-proxy")
                   :clone-from gold-zone
                   (zone/fs "/home" :special "/export/home")
                   (network "serv-proxy")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-proxy.janet")))

      (zone/ensure "serv-records"
                   :brand "pkgsrc"
                   :recreate (recreate? "serv-records")
                   (zone/fs "/home" :special "/export/home")
                   (network "serv-records")
                   :dns globals/zone-dns
                   (zone/bootstrap :file (pathcat gurp-dir "zone-records.janet"))))

(host "serv" (zones))
