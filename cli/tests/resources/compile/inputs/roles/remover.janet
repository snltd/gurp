(role remover
      # An artificial role that exercises most of the  /remove doers

      # cron/remove
      # (directory/remove "/") # Protected directory
      (directory/remove "/var/krb5")
      (directory/remove "/var/yp/binding")
      (directory/remove "/never/existed")

      # (file/remove "/etc/passwd") # Protected file
      (file/remove "/var/yp/aliases")
      (file/remove "/var/yp/nicknames")
      (file/remove "/never/existed")

      # # file-line/remove

      # # gem/remove - will write a proper gem tester

      (group/remove "gdm")
      (group/remove "upnp")
      (group/remove "never-existed")

      (pkg/remove "compress/unzip")
      (pkg/remove "never/existed")

      # pkgin/remove - not a pkgsrc zone

      (publisher/remove "extra.omnios")
      (publisher/remove "never.existed")

      (smf/remove "svc:/network/ssh:default")
      (smf/remove "svc:/never/existed")

      # svcprop/remove

      (symlink/remove "/var/ld/64")
      (symlink/remove "/never/existed")

      # (user/remove "sys") # Protected user
      (user/remove "zfssnap")
      (user/remove "upnp")
      (user/remove "never-existed")

      # zone/remove - exercised by the main test script
      # zfs/remove - real removals are exercised by the main test script
      (zfs/remove "never/existed"))
