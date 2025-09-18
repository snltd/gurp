(use ../globals)

(def smf-method-path (pathcat site-smf-method "minidlna.sh"))
(def smf-service "sysdef/multimedia/minidlna")

(role minidlna
      (def minidlna-conf
        [:media_dir "A,/storage/flac"
         :media_dir "A,/storage/mp3"
         :friendly_name (hostname)
         :album_art_names "front.jpg"
         :album_art_names "front.png"
         :inotify "yes"
         :strict_dlna "no"
         :notify_interval 900])

      (pkg/ensure "ooce/multimedia/minidlna")

      (file/ensure smf-method-path
                   :mode "0755"
                   :from "minidlna/minidlna-method.sh")

      (file/ensure "/etc/opt/ooce/minidlna/minidlna.conf"
                   :from-struct minidlna-conf
                   :to-format "k=v")

      (svc/ensure smf-service :state "online")

      (smf/ensure smf-service
                  :fmri smf-service
                  :description "MiniDLNA - DLNA/UPnP-AV media server"
                  (smf-method "start"
                              :exec smf-method-path
                              :user "minidlna"
                              :group "minidlna")
                  (smf-method "refresh"
                              :exec smf-method-path
                              :user "minidlna"
                              :group "minidlna")))
