(use judge
     ./lib/test-lib
     ../../src/collector)
(import ../../src/doers/ipmp)
(import ../../src/doers/ip-interface)

(deftest ipmp
  (set *collector* (new-collector))
  (setdyn :role-dyn "test-role")

  (import-tests "ipmp")

  (test *collector*
    @{:ensure @{"ipmp" @[{:_id "/test-role/ipmp/ipmp0"
                          :interfaces ["mh0" "mh1" "mh2"]
                          :name "ipmp0"
                          :role "test-role"}]
                :ip-interface @[{:_id "/test-role/ip-interface/mh0"
                                 :name "mh0"
                                 :role "test-role"}
                                {:_id "/test-role/ip-interface/mh1"
                                 :name "mh1"
                                 :role "test-role"}
                                {:_id "/test-role/ip-interface/mh2"
                                 :name "mh2"
                                 :role "test-role"}]}
      :remove @{"ipmp" @[{:_id "/test-role/ipmp/ipmp1"
                          :name "ipmp1"
                          :role "test-role"}]}}))

(deftest ipmp-error
  (test-error (ipmp/ensure "ipmp2")
              "In ipmp/ensure ipmp2: did not find mandatory property :interfaces. Mandatory properties are :interfaces")
  (test-error (ipmp/ensure "ipmp3" :interfaces ["bge1" "bge2"] :wat "oops")
              "In ipmp/ensure ipmp3: unexpected property :wat. Valid properties are :interfaces, :label")
  (test-error (ipmp/remove "ipmp4" :wat "oops")
              "In ipmp/remove ipmp4: unexpected property :wat. Valid properties are :label"))
