(use judge)
(use ../lib/gurp)

(deftest "group-resources"
  (set *collector* (new-collector))

  (setdyn :role-dyn "test-role")
  (group/ensure "new-group" :gid 264)

  (group/remove "old-group")

  (test *collector*
    @{:ensure @{:group @[{:_id "/test-role/group/new-group"
                          :gid 264
                          :name "new-group"
                          :role "test-role"}]}
      :remove @{:group @[{:_id "/test-role/group/old-group"
                          :name "old-group"
                          :role "test-role"}]}}))

(deftest "group-errors"
  (test-error
    (group/ensure "wat")
    "Failed to validate user input for group 'wat': group missing required key(s): gid")

  (test-error
    (group/ensure "group"
                 :gid 264
                 :gecos "Test User")
    "Failed to validate user input for group 'group': group 'group' has unrecognised key(s): gecos"))
