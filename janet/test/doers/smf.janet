(use judge)
(use ./_helpers)
(use ../../src/collector)
(import ../../src/doers/smf)

(deftest smf
  (set *collector* (new-collector))

  (import-tests "smf" (curenv))

  (test *collector*
    @{:ensure @{:smf @[{:_id "/NO-ROLE/smf/telegraf"
                        :default-enabled true
                        :dependencies @[@{:fmri "svc://example/service1:default"
                                          :grouping "require_all"
                                          :name "svc1"
                                          :restart-on "none"
                                          :type "service"}
                                        @{:fmri "svc://example/service2:default"
                                          :grouping "optional-all"
                                          :name "svc2"
                                          :restart-on "error"
                                          :type "service"}]
                        :description "Run Telegraf agent"
                        :fmri "sysdef/telegraf"
                        :name "telegraf"
                        :properties @{:application/datadir {:type "astring" :value "/data"}}
                        :property-groups {:application "application"}
                        :role "NO-ROLE"
                        :single-instance true
                        :start-method @{:context {:group "daemon"
                                                  :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"
                                                  :user "telegraf"}
                                        :exec "/opt/site/lib/smf/method/telegraf.sh"
                                        :timeout 60}
                        :stop-method {:exec ":kill" :timeout 10}}]}
      :remove @{:smf @[{:_id "/NO-ROLE/smf/some_unwanted_service"
                        :name "some/unwanted/service"
                        :role "NO-ROLE"}]}}))

(test-error
  (smf/ensure "telegraf"
              :description "Run Telegraf agent"
              :fmri "sysdef/telegraf"
              (smf/method "start" :exec "/opt/site/lib/smf/method/telegraf.sh")
              (smf/method "refresh" :exec ":kill -THAW")
              (smf/method "gibbus" :exec "gibbus"))
  "smf/method name must be one of \"start\", \"stop\", \"refresh\", \"reload\"")

(test-error
  (smf/ensure "telegraf"
              (smf/method "start"
                          :exec "/opt/site/lib/smf/method/telegraf.sh"
                          :user "telegraf"
                          :group "daemon"))
  "did not find mandatory property :fmri. Mandatory properties are :fmri")
