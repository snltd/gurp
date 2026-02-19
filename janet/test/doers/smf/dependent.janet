(use judge)
(import ../../../src/doers/smf)

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
    "In smf/dependent svc1: did not find mandatory property :fmri. Mandatory properties are :name, :fmri")
  (test-error
    (smf/dependent "svc1" :fmri "svc://test/svc1:default" :junk "junk")
    "In smf/dependent svc1: unexpected property :junk. Valid properties are :name, :fmri, :grouping, :type, :restart-on, :label"))
