(use roles/basenode)
(use roles/cron-runner-user)
(use roles/www-records)

(host "www-records"
  (basenode)
  (cron-runner-user)
  (www-records))
