(host "tester"
  :vars {
    :var_a "value_a"
    :var_b "value_b"
  }
  :modules [
    "physical"
    "common"
    "omnios_server"
    "file_store"
    "samba"
    "telegraf"
    "zfs_snapshot"
  ])
