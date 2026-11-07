(use judge)
(import ../../../src/doers/zone)

(deftest zone/rctl
  (test
    (zone/rctl "zone.max-physical-memory"
               :limit 524288000)
    [:rctl
     @{:action "deny"
       :from-helper true
       :limit 524288000
       :name "zone.max-physical-memory"
       :priv "privileged"
       :unique false}])

  (test
    (zone/rctl "zone.max-physical-memory"
               :action "allow"
               :limit 12345678)
    [:rctl
     @{:action "allow"
       :from-helper true
       :limit 12345678
       :name "zone.max-physical-memory"
       :priv "privileged"
       :unique false}])

  (test-error
    (zone/rctl "zone.max-physical-memory"
               :limit 524288000
               :oops "wat?")
    "In zone/rctl zone.max-physical-memory: unexpected property :oops. Valid properties are :action, :label, :limit, :priv")

  (test-error
    (zone/rctl "zone.max-physical-memory")
    "In zone/rctl zone.max-physical-memory: did not find mandatory property :limit. Mandatory properties are :action, :limit, :priv"))
