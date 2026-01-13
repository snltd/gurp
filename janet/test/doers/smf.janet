(use judge)
(use ../../src/collector)
(import ../../src/doers/smf)

(deftest method
  (test
    (smf/method "start"
                :exec "/opt/site/lib/smf/method/telegraf.sh"
                :user "telegraf"
                :group "daemon"
                :privileges ["basic" "file_dac_search" "sys_admin"
                             "proc_owner" "proc_zone"])
    {:start-method @{:context {:group "daemon"
                               :privileges "basic,file_dac_search,sys_admin,proc_owner,proc_zone"
                               :user "telegraf"}
                     :exec "/opt/site/lib/smf/method/telegraf.sh"
                     :timeout 60}}))

(deftest dependency
  (test
    (smf/dependency "svc1" :fmri "svc://test/svc1:default")
    {:dependencies @{:fmri "svc://test/svc1:default"
                     :grouping "require_all"
                     :name "svc1"
                     :restart-on "none"
                     :type "service"}})
  (test
    (smf/dependency "svc2"
                    :grouping "optional-all"
                    :restart-on "error"
                    :fmri "svc://example/service2:default")
    {:dependencies @{:fmri "svc://example/service2:default"
                     :grouping "optional-all"
                     :name "svc2"
                     :restart-on "error"
                     :type "service"}})

  (test-error
    (smf/dependency "svc1" :service "svc://test/svc1:default")
    "did not find mandatory property :fmri. Mandatory properties are :name, :fmri")
  (test-error
    (smf/dependency "svc1" :fmri "svc://test/svc1:default" :junk "junk")
    "unexpected property :junk. Valid properties are :name, :fmri, :grouping, :type, :restart-on, :label"))

(deftest dependent
  (test
    (smf/dependent "svc1" :fmri "svc://test/svc1:default")
    {:dependencies @{:fmri "svc://test/svc1:default"
                     :grouping "require_all"
                     :name "svc1"
                     :restart-on "none"
                     :type "service"}})
  (test
    (smf/dependent "svc2"
                   :grouping "optional-all"
                   :restart-on "error"
                   :fmri "svc://example/service2:default")
    {:dependencies @{:fmri "svc://example/service2:default"
                     :grouping "optional-all"
                     :name "svc2"
                     :restart-on "error"
                     :type "service"}})

  (test-error
    (smf/dependent "svc1" :service "svc://test/svc1:default")
    "did not find mandatory property :fmri. Mandatory properties are :name, :fmri")
  (test-error
    (smf/dependent "svc1" :fmri "svc://test/svc1:default" :junk "junk")
    "unexpected property :junk. Valid properties are :name, :fmri, :grouping, :type, :restart-on, :label"))

(deftest "smf-resources"
  (set *collector* (new-collector))

  (smf/ensure "telegraf"
              :description "Run Telegraf agent"
              :fmri "sysdef/telegraf"
              (smf/dependency "svc1"
                              :fmri "svc://example/service1:default")
              (smf/dependency "svc2"
                              :grouping "optional-all"
                              :restart-on "error"
                              :fmri "svc://example/service2:default")
              (smf/method "start"
                          :exec "/opt/site/lib/smf/method/telegraf.sh"
                          :user "telegraf"
                          :group "daemon"
                          :privileges ["basic" "file_dac_search" "sys_admin"
                                       "proc_owner" "proc_zone"])
              :property-groups {:application "application"}
              :properties {:application/datadir "/data"})

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
          :remove @{}}))

(deftest "smf-error"
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
    "did not find mandatory property :fmri. Mandatory properties are :fmri"))

