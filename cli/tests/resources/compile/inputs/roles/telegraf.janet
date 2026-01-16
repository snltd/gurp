(import ../globals)

(def svc "sysdef/telegraf")
(def executable (pathcat globals/site-bin "telegraf"))
(def conf (pathcat globals/site-etc "telegraf.conf"))
(def method (pathcat globals/site-smf-method "telegraf.sh"))

(indoc method-content `
  #!/bin/ksh
  
  . /lib/svc/share/smf_include.sh
  
  {{ executable }} --config {{ conf }} &
  
  exit $SMF_EXIT_OK
  `)

(role telegraf
      (user/ensure "telegraf"
                   :gecos "Telegraf pseudo-user"
                   :home-dir "/var/tmp"
                   :uid 108
                   :primary-group "daemon"
                   :shell "/bin/false")

      (file/ensure executable
                   :label "executable"
                   :from "telegraf/telegraf"
                   :mode "0755")

      (file/ensure conf
                   :label "conf"
                   :from "telegraf/telegraf.conf.physical")

      (file/ensure method
                   :mode "0755"
                   :content (template-out method-content {:executable executable
                                                          :conf conf}))

      (svc/ensure svc
                  :state "online"
                  :restarted-by [(this :file "conf")
                                 (this :file "executable")])

      (smf/ensure "telegraf"
                  :description "Run Telegraf agent"
                  :fmri svc
                  (smf/method "start"
                              :exec method
                              :user "telegraf"
                              :group "daemon"
                              :privileges ["basic" "file_dac_search" "sys_admin" "proc_owner" "proc_zone"])))
