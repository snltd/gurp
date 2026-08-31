(use judge)
(use ./test-lib)
(use ../../src/collector)
(import ../../src/doers/svcprop)

(deftest svcprop
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "svcprop")

  (test *collector*
    @{:ensure @{:svcprop @[{:_id "/test-role/svcprop/example_svc_1"
                            :name "example/svc_1"
                            :on-change "restart"
                            :properties @{:application/active {:type "boolean" :value true}
                                          :application/datadir {:type "astring" :value "/data"}
                                          :application/timeout {:type "integer" :value 50}}
                            :property-groups {:application "application"}
                            :role "test-role"}
                           {:_id "/test-role/svcprop/example_svc_1"
                            :name "example/svc_1"
                            :properties @{:application/datadir {:type "astring" :value "/data"}}
                            :property-groups {:application "application"}
                            :role "test-role"}]}
      :remove @{:svcprop @[{:_id "/test-role/svcprop/example_svc_3"
                            :name "example/svc_3"
                            :properties ["application/thing"]
                            :role "test-role"}]}}))

(deftest svcprop-error
  (test-error
    (svcprop/ensure "mariadb"
                    :properties {:application/active true}
                    :on-change "explode")
    "In svcprop/ensure mariadb: on-change action must be one of restart, refresh [got 'explode']")

  (test-error
    (svcprop/ensure "mariadb" :wat true)
    "In svcprop/ensure mariadb: did not find mandatory property :properties. Mandatory properties are :properties"))
