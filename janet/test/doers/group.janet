(use judge)
(use ../../src/collector)
(import ../../src/doers/group)

(deftest "group-resources"
  (set *collector* (new-collector))

  (setdyn :role-dyn "test-role")
  (group/ensure "new-group" :gid 264)

  (group/remove "old-group")

  (test *collector*
    @{:ensure @{:group @[@{:_id "/test-role/group/new-group"
                           :gid 264
                           :name "new-group"
                           :role "test-role"}]}
      :remove @{:group @[@{:_id "/test-role/group/old-group"
                           :name "old-group"
                           :role "test-role"}]}}))

(deftest "group-errors"
  (test-error
    (group/ensure "wat")
    "did not find mandatory property 'gid'. Mandatory propties are: gid")

  (test-error
    (group/ensure "group"
                 :gid 264
                 :gecos "Test User")
    "unexpected property 'gecos'. Valid properties are: gid, label"))
