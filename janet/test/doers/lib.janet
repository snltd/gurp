(use judge)
(use ../../src/doers/lib)

(def optional-keys {})

(def mandatory-keys
  {:group {:types [:string :number]
           :help "The group name or GID of the for this directory"}
   :mode {:types [:string]
          :help "Permissions, written as a four-digit octal"}
   :owner {:types [:string :number]
           :help "The username or UID of the user who owns this directory"}})

(deftest test-checked-spec-with-directory-ok-names
  (test
    (checked-spec
      {:group "sysadmin" :mode "0755" :owner "rob"} mandatory-keys optional-keys)
    {:group "sysadmin"
     :mode "0755"
     :owner "rob"}))

(deftest test-checked-spec-with-directory-ok-numbers
  (test
    (checked-spec
      {:group 14 :mode "0755" :owner 264} mandatory-keys optional-keys)
    {:group 14 :mode "0755" :owner 264}))

(deftest test-checked-spec-with-directory-incorrect-mandatory-type
  (test-error
    (checked-spec
      {:group 14 :mode 755 :owner 264} mandatory-keys optional-keys)
    "mode is of type :number. Allowed types :string"))

(deftest test-checked-spec-with-directory-unknown-optional
  (test-error
    (checked-spec
      {:group 14 :mode "0755" :owner 264 :oops "merp"} mandatory-keys optional-keys)
    "unexpected property :oops. Valid properties are :owner, :group, :mode, :label"))

(deftest test-check-key-type
  (test (check-key-type :name 123 [:number :string]) nil)
  (test (check-key-type :name "merp" [:number :string]) nil)

  (test-error
    (check-key-type :name 123 [:string])
    "name is of type :number. Allowed types :string"))

(deftest make-spec-struct
  (test (make-spec-struct :a 1 :b 2 :c 3) {:a 1 :b 2 :c 3})
  (test (make-spec-struct []) {})
  (test (make-spec-struct) {})

  (test-error
    (make-spec-struct :a 1 :b)
    "unable to create struct from (:a 1 :b): expected even number of arguments"))

(deftest spec-with-defaults
  (test (spec-with-defaults {} {}) @{})
  (test (spec-with-defaults {:a 1 :b 2} {:b 10 :c 20}) @{:a 1 :b 10 :c 20}))

(deftest spec->resource
  (test (spec->resource
          :directory "/tmp/testdir" {:owner "rob" :group "sysadmin" :mode "0700"})
    @{:_id "/NO-ROLE/directory/_tmp_testdir"
      :group "sysadmin"
      :mode "0700"
      :name "/tmp/testdir"
      :owner "rob"
      :role "NO-ROLE"}))
