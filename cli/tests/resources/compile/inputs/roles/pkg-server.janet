(import ../globals)
(import ../helpers)

(indoc startup-method-template `
  #!/bin/sh -e
  # 
  if ! test -d "{{ repo-root}}/publisher"
  then
	  /usr/bin/pkgrepo create {{ repo-root }}
  fi
  
	/usr/bin/pkgrepo set -s {{ repo-root }} publisher/prefix={{ repo-name }}
  {{ refresh-repo-script }}
`)

(indoc refresh-repo-template `
  #!/bin/ksh -e

  REPO_NAME="{{ repo-name }}"
  REPO_ROOT="{{ repo-root }}"
  REPO_SVC="{{ repo-svc }}"
  MARKER="/var/run/refresh_repo"

  test -f $MARKER || /bin/touch $MARKER

  if [[ $1 != "-f" ]]
  then
    if ! test "${REPO_ROOT}/publisher/${REPO_NAME}/pkg" -nt $MARKER
    then
      /bin/touch $MARKER
      exit 0
    fi
  fi

  /usr/bin/pkgrepo refresh -s $REPO_ROOT refresh 
  # /usr/sbin/svcadm refresh $REPO_SVC
  # /usr/sbin/svcadm restart $REPO_SVC
  /bin/touch $MARKER
`)

(def repo-name "sysdef")
(def startup-method (pathcat globals/site-smf-method (string repo-name "-repo-setup.sh")))
(def startup-svc (string "sysdef/application/" repo-name "-setup"))
(def repo-svc (string "application/pkg/server:" repo-name))
(def repo-root "/repo")
(def pkg-log-dir "/var/log/pkg")
(def refresh-repo-script (pathcat globals/site-bin "refresh-pkg-repo"))

(role pkg-server
      (zfs/ensure (zfscat globals/fast-pool "zone"))
      (zfs/ensure (zfscat globals/fast-pool "zone" "pkg"))
      (zfs/ensure (zfscat globals/fast-pool "zone" "pkg" "repo")
                  :properties {:mountpoint repo-root})

      (directory/ensure pkg-log-dir
                        :owner "pkg5srv"
                        :group "daemon")

      (directory/ensure (pathcat pkg-log-dir "server")
                        :owner "pkg5srv"
                        :group "daemon")

      (directory/ensure repo-root
                        :owner "pkg5srv"
                        :group "pkg5srv")

      (section init-repo
               (smf/ensure startup-svc
                           :fmri startup-svc
                           :description "transient service to create pkg repo"
                           :duration "transient"
                           (smf-method "start" :exec startup-method))

               (file/ensure startup-method
                            :mode "0755"
                            :content (template-out startup-method-template
                                                   {:repo-name repo-name
                                                    :repo-root repo-root
                                                    :refresh-repo-script refresh-repo-script})))

      (section auto-refresh
               (user/ensure "pkg5srv"
                            :uid 97
                            :primary-group "pkg5srv"
                            :gecos "pkg(7) server"
                            :home-dir "/"
                            :shell ""
                            :password-hash "NP")

               (file/ensure refresh-repo-script
                            :mode "0755"
                            :content (template-out refresh-repo-template
                                                   {:repo-name repo-name
                                                    :repo-root repo-root
                                                    :repo-svc repo-svc}))

               (cron/ensure "refresh-pkg-repo"
                            :minute "*/5"
                            :user "pkg5srv"
                            :command (helpers/site-cron "refresh-pkg-repo")))

      (svcprop/ensure repo-svc
                      :property-groups {:pkg "application"}
                      :properties {:pkg/inst_root repo-root
                                   :pkg/readonly false
                                   :pkg/log_errors (pathcat pkg-log-dir "error.log")
                                   :pkg/log_access (pathcat pkg-log-dir "access.log")})

      (svc/ensure repo-svc :state "online"))
