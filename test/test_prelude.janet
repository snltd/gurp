(use judge)
(use ../prelude)

(def test-uid->user (create-uid-map "test/resources/passwd" 2 7))
(def test-user->uid (invert test-uid->user))
(def test-gid->group (create-uid-map "test/resources/group" 1 4))
(def test-group->gid (invert test-gid->group))

(test (get test-uid->user 3))
(test (get test-user->uid "daemon"))
(test (get test-user->uid "wat?"))
(test (get test-gid->group 3))
(test (get test-group->gid "sysadmin"))
(test (get test-group->gid "wat?"))
