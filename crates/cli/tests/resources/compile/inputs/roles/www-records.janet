(import ../globals)
(import ../helpers)
(import ../secrets)

(def app-name "record-collector")
(def svc-name (string "sysdef/sinatra/" app-name))

(role www-records
      (def app-log-dir (pathcat "/var/log" app-name))

      (def app-conf
        {:mysql_creds {:adapter "mysql"
                       :host "mysql"
                       :username "rec"
                       :password secrets/www-records-password
                       :database "records"
                       :reconnect true
                       :encoding "utf8"}
         :discogs_token secrets/discogs-token
         :metric_base (string "dev." app-name)
         :logdir app-log-dir})

      (pkgin/ensure "ruby33-nokogiri")
      (pkgin/ensure "ruby33-puma")

      (gem/ensure app-name
                  :gem-path "/opt/local/bin/gem"
                  :source (string/format "https://snltd:%s@rubygems.pkg.github.com/snltd"
                                         secrets/github-pat))

      (directory/ensure app-log-dir
                        :owner "sinatra"
                        :group "daemon"
                        :mode "0775")

      (file/ensure (pathcat globals/site-etc (string app-name ".yml"))
                   :from-struct app-conf
                   :to-format "yaml")

      (user/ensure "sinatra"
                   :gecos "Sinatra user"
                   :primary-group "daemon"
                   :shell "/bin/false"
                   :home-dir "/var/tmp"
                   :uid 4567)

      (smf/ensure app-name
                  :fmri svc-name
                  :description "record collection web-app"
                  :properties {:restarter/contract "fixed"
                               :restarter/count 10
                               :restarter/delay 10}
                  (smf/method "start"
                              :exec (string/format
                                      "%s rackup -E prod -D $(%s)"
                                      (pathcat globals/gem-dir "rackup")
                                      (pathcat globals/gem-dir (string "locate_" app-name)))
                              :user "sinatra"
                              :group "nogroup"
                              :privileges ["basic" "!proc_session" "!proc_info" "!file_link_any"]
                              :environment {:LC_CTYPE "en_US.UTF-8"
                                            :PATH (string globals/gem-dir ":/opt/local/bin:/bin")}))

      (svc/ensure svc-name :state "online")

      (cron/ensure "discogs-price-update"
                   :command (string "/opt/ooce/bin/discogs-price-update "
                                    ">"
                                    (pathcat globals/cron-log-dir "discogs-price-update.log"))
                   :minute "19,49"
                   :user "cron"))
