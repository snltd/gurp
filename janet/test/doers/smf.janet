(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/smf)

(deftest smf
  (set *collector* (new-collector))

  (import-tests "smf" (curenv))

  (test *collector*
    @{:ensure @{:smf @[{:_id "/NO-ROLE/smf/example"
                        :default-enabled true
                        :dependencies @[@{:fmri "svc:/milestone/name-services:default"
                                          :grouping "require_all"
                                          :name "dependency1"
                                          :restart-on "none"
                                          :type "service"}
                                        @{:fmri "svc:/system/pkgserv:default"
                                          :grouping "optional_all"
                                          :name "dependency2"
                                          :restart-on "error"
                                          :type "service"}]
                        :description "Run example program"
                        :duration "child"
                        :fmri "snltd/example"
                        :name "example"
                        :properties @{:application/datadir {:type "astring" :value "/data"}}
                        :property-groups {:application "application"}
                        :role "NO-ROLE"
                        :single-instance true
                        :start-method @{:context {:group "daemon"
                                                  :privileges "basic,!file_dac_search"
                                                  :user "appuser"}
                                        :exec "/app/method.sh"
                                        :timeout 60}
                        :stop-method {:exec ":kill" :timeout 10}}]}
      :remove @{:smf @[{:_id "/NO-ROLE/smf/unwanted_service"
                        :name "unwanted/service"
                        :role "NO-ROLE"}]}}))

(deftest smf-error
  (test-error
    (smf/ensure "telegraf"
                :description "Run Telegraf agent"
                :fmri "sysdef/telegraf"
                (smf/method "start" :exec "/opt/site/lib/smf/method/telegraf.sh")
                (smf/method "refresh" :exec ":kill -THAW")
                (smf/method "gibbus" :exec "gibbus"))
    "In smf/method gibbus: smf/method name must be one of \"start\", \"stop\", \"refresh\", \"reload\"")

  (test-error
    (smf/ensure "telegraf"
                (smf/method "start"
                            :exec "/opt/site/lib/smf/method/telegraf.sh"
                            :user "telegraf"
                            :group "daemon"))
    "In smf/ensure telegraf: did not find mandatory property :fmri. Mandatory properties are :fmri"))
