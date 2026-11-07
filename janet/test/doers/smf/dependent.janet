(use judge)
(import ../../../src/doers/smf)

(deftest dependent
  (test
    (smf/dependent "svc1" :fmri "svc://test/svc1:default")
    [:dependent
     @{:fmri "svc://test/svc1:default"
       :from-helper true
       :grouping "require_all"
       :name "svc1"
       :restart-on "none"
       :type "service"
       :unique false}])
  (test
    (smf/dependent "svc2"
                   :grouping "optional-all"
                   :restart-on "error"
                   :fmri "svc://example/service2:default")
    [:dependent
     @{:fmri "svc://example/service2:default"
       :from-helper true
       :grouping "optional-all"
       :name "svc2"
       :restart-on "error"
       :type "service"
       :unique false}])

  (test-error
    (smf/dependent "svc1" :service "svc://test/svc1:default")
    "In smf/dependent svc1: did not find mandatory property :fmri. Mandatory properties are :fmri, :grouping, :restart-on, :type")
  (test-error
    (smf/dependent "svc1" :fmri "svc://test/svc1:default" :junk "junk")
    "In smf/dependent svc1: unexpected property :junk. Valid properties are :fmri, :grouping, :label, :restart-on, :type"))
