(use roles/basenode)
(use roles/file-store)
(use roles/physical)
(use roles/telegraf)
(use roles/zfs-snapshot)
(use roles/cron-monitor)

(host "dev-server"
      (physical)
      (basenode)
      (telegraf)
      (file-store)
      (cron-monitor)
      (zfs-snapshot))
