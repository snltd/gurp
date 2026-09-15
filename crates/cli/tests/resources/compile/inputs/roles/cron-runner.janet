(role cron-runner
      (pkg/ensure "ooce/runtime/ruby-34")
      (pkg/ensure "ooce/library/yaml")
      (pkg/ensure "network/netcat"))

# (section pch-reboot
#          (cron/ensure "reboot PCH"
#                       :hour 0
#                       :minute 58
#                       :user "cron"
#                       :command "echo -e "/sbin/reboot\\r\\n" | nc pch.lan.id264.net 23"

