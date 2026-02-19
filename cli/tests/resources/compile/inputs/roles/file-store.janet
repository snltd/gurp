(import ../globals)
(import ../helpers)
(import ../secrets)

(def home-root "/export/home")

(def sharenfs
  {:plain (string/format "root=@%s/32,rw=@192.168.1.0/24" (helpers/ip-of "slim"))
   :equivalent-root (string/format "root=@%s/32,rw=@%s,ro=@192.168.1.0/24"
                                   (helpers/ip-of "slim")
                                   (helpers/ip-of "slim"))})

(defn zfs-share-nfs [pool dataset share-type]
  (zfs/ensure (zfscat pool dataset)
              :properties {:mountpoint (pathcat "/export" dataset)
                           :sharenfs (sharenfs share-type)
                           :compression "lz4"
                           :devices "off"
                           :exec "off"
                           :setuid "off"
                           :atime "off"}))

(defn zfs-share-nfs-smb [pool dataset]
  (zfs/ensure (zfscat pool dataset)
              :properties {:mountpoint (pathcat "/export" dataset)
                           :sharenfs (sharenfs :plain)
                           :sharesmb (string "name=" (last (string/split "/" dataset)))
                           :compression "lz4"
                           :devices "off"
                           :exec "off"
                           :setuid "off"
                           :atime "off"}))

(defn zfs-placeholder [pool dataset]
  (zfs/ensure (zfscat pool dataset)
              :properties {:mountpoint "none"
                           :compression "lz4"}))

(role file-store
      (link/ensure "/home" :source home-root)

      (section users
               (zfs/ensure (zfscat globals/fast-pool "export/home/rob")
                           :properties {:mountpoint "/export/home/rob"
                                        :sharenfs (sharenfs :plain)
                                        :sharesmb "off"
                                        :compression "lz4"
                                        :devices "off"
                                        :exec "on"
                                        :setuid "off"
                                        :atime "off"})

               (zfs/ensure (zfscat globals/fast-pool "export/home/rob/work")
                           :properties {:mountpoint "/export/home/rob/work"
                                        :sharenfs (sharenfs :plain)
                                        :sharesmb "off"
                                        :compression "lz4"
                                        :devices "off"
                                        :exec "on"
                                        :setuid "off"
                                        :atime "off"})

               (zfs-share-nfs globals/big-pool "user_data/rob" :equivalent-root)
               (misc/ensure :enable-smb "rob")

               (user/ensure "klf"
                            :shell "/bin/zsh"
                            :primary-group "staff"
                            :gecos "klf"
                            :home-dir (pathcat home-root "klf")
                            :password-hash "some-hash-or-other"
                            :uid 266)

               (zfs/ensure (zfscat globals/fast-pool "export/home/klf")
                           :properties {:mountpoint "/export/home/klf"
                                        :sharenfs (sharenfs :plain)
                                        :sharesmb "name=klf"
                                        :compression "lz4"
                                        :devices "off"
                                        :exec "off"
                                        :setuid "off"
                                        :atime "off"})

               (zfs-share-nfs globals/big-pool "user_data/klf" :equivalent-root)
               (misc/ensure :enable-smb "klf"))

      (section rpool
               (zfs/ensure "rpool/zones" :properties {:mountpoint "/zones"}))

      (section big-pool
               (zfs/ensure (zfscat globals/big-pool))
               (zfs/ensure (zfscat globals/big-pool "zone"))
               (zfs/ensure (zfscat globals/big-pool "bhyve"))

               (zfs-share-nfs-smb globals/big-pool "video")
               (zfs-share-nfs-smb globals/big-pool "flac")

               (zfs-share-nfs globals/big-pool "software" :equivalent-root)
               (zfs-share-nfs globals/big-pool "fonts" :equivalent-root)
               (zfs-share-nfs globals/big-pool "games" :equivalent-root)
               (zfs-share-nfs globals/big-pool "sysdef" :equivalent-root)
               (zfs-share-nfs globals/big-pool "kronos" :equivalent-root)

               (zfs/ensure (zfscat globals/big-pool "export/pkg_repo")
                           :properties {:mountpoint "/export/pkg_repo"
                                        :compression "lz4"
                                        :devices "off"
                                        :exec "off"
                                        :setuid "off"}))

      (section fast-pool
               (zfs/ensure (zfscat globals/fast-pool))
               (zfs/ensure (zfscat globals/fast-pool "zone"))
               (zfs/ensure (zfscat globals/fast-pool "user_data"))
               (zfs/ensure (zfscat globals/fast-pool "export"))
               (zfs/ensure (zfscat globals/fast-pool "export/home"))

               (zfs-share-nfs-smb globals/fast-pool "mp3")
               (zfs-share-nfs-smb globals/fast-pool "photos")))
