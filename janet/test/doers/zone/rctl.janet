(use judge)
(import ../../../src/doers/zone)

(deftest zone/rctl
  (test
    (zone/rctl "zone.max-physical-memory"
               :limit 524288000)
    {:rctl @{:action "deny"
             :limit 524288000
             :name "zone.max-physical-memory"
             :priv "privileged"}})

  (test
    (zone/rctl "zone.max-physical-memory"
               :action "allow"
               :limit 12345678)
    {:rctl @{:action "allow"
             :limit 12345678
             :name "zone.max-physical-memory"
             :priv "privileged"}})

  (test-error
    (zone/rctl "zone.max-physical-memory")
    "did not find mandatory property :limit. Mandatory properties are :priv, :name, :action, :limit"))
