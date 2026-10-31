(use judge)
(use ../../src/doers/lib)

(def optional-keys {})

(deftest expand-svc-property
  (test
    (expand-svc-property {:application/active true
                                         :application/datadir "/data"
                                         :application/timeout 50})
    @{:application/active {:type "boolean" :value true}
      :application/datadir {:type "astring" :value "/data"}
      :application/timeout {:type "integer" :value 50}})
)

(comment
(def mandatory-keys
  {:group {:types [:string :number]
           :help "The group name or GID of the for this directory"}
   :mode {:types [:string]
          :help "Permissions, written as a four-digit octal"}
   :owner {:types [:string :number]
           :help "The username or UID of the user who owns this directory"}})

(deftest comma-sep
  (test (comma-sep [:merp :byerp :gurp]) ":merp, :byerp, :gurp")
  (test (comma-sep ["merp" "byerp" "gurp"]) "\"merp\", \"byerp\", \"gurp\"")
  (test (comma-sep ["merp"]) "\"merp\"")
  (test (comma-sep []) ""))

(deftest check-key-type
  (test (check-key-type :name 123 [:number :string]) nil)
  (test (check-key-type :name "merp" [:number :string]) nil)
  (test-error
    (check-key-type :name 123 [:string])
    "name is of type :number. Allowed types :string"))

(deftest make-spec-struct
  (test (make-spec-struct :a 1 :b 2 :c 3) {:a 1 :b 2 :c 3})
  (test (make-spec-struct ;[]) {})
  (test (make-spec-struct) {})
  (test-error
    (make-spec-struct :a 1 :b)
    "unable to create struct from 3 arg(s):  (:a 1 :b): expected even number of arguments"))

(deftest checked-spec-with-directory-ok-names
  (test
    (checked-spec
      {:group "sysadmin" :mode "0755" :owner "rob"} mandatory-keys optional-keys)
    {:group "sysadmin"
     :mode "0755"
     :owner "rob"}))

(deftest checked-spec-with-directory-ok-numbers
  (test
    (checked-spec
      {:group 14 :mode "0755" :owner 264} mandatory-keys optional-keys)
    {:group 14 :mode "0755" :owner 264}))

(deftest checked-spec-with-directory-incorrect-mandatory-type
  (test-error
    (checked-spec
      {:group 14 :mode 755 :owner 264} mandatory-keys optional-keys)
    "mode is of type :number. Allowed types :string"))

(deftest checked-spec-with-directory-unknown-optional
  (test-error
    (checked-spec
      {:group 14 :mode "0755" :owner 264 :oops "merp"} mandatory-keys optional-keys)
    "unexpected property :oops. Valid properties are :group, :label, :mode, :owner"))

(deftest checked-spec-with-invalid-value
  (def mandatory-keys
    {:synth {:types [:string]
             :help "Which synth?"
             :values ["kronos" "blofeld"]}})

  (test
    (checked-spec {:synth "blofeld"} mandatory-keys {})
    {:synth "blofeld"})

  (test-error
    (checked-spec {:synth "pro3"} mandatory-keys {})
    "synth must be one of \"kronos\", \"blofeld\" [Got 'pro3']"))

(deftest checked-spec-with-struct-values
  (def mandatory-keys
    {:from {:types [:string]
            :help "Path or URL to resource"}})
  (def optional-keys
    {:checksum {:types [:struct]
                :help "Define a checksum"
                :definition {:algorithm {:types [:string]
                                         :help "Hashing algorithm"
                                         :values ["sha512" "blake3"]}
                             :type {:types [:string]
                                    :help "Type of value"
                                    :values ["suffix" "url" "literal"]}
                             :value {:types [:string]
                                     :help "Checksum, suffix, or url"}}}})

  (test
    (checked-spec {:from "/path/to/file"
                   :checksum {:algorithm "sha512"
                              :type "suffix"
                              :value ".sha256"}} mandatory-keys optional-keys)
    {:checksum {:algorithm "sha512"
                :type "suffix"
                :value ".sha256"}
     :from "/path/to/file"})

  (test-error
    (checked-spec {:from "/path/to/file"
                   :checksum {:algorithm "des"
                              :type "suffix"
                              :value ".sha256"}} mandatory-keys optional-keys)
    "algorithm must be one of \"sha512\", \"blake3\" [Got 'des']"))

(deftest resource-id
  (test (resource-id :file :test-role :merp) "/test-role/file/merp")
  (test (resource-id :file :test-role nil) "/test-role/file/NO-NAME"))

(deftest spec->resource
  (test (spec->resource
          :directory "/tmp/testdir" {:owner "rob" :group "sysadmin" :mode "0700"})
        {:_id "/NO-ROLE/directory/_tmp_testdir"
         :group "sysadmin"
         :mode "0700"
         :name "/tmp/testdir"
         :owner "rob"
         :role "NO-ROLE"}))

(deftest spec-with-defaults
  (test (spec-with-defaults {} {}) @{})
  (test (spec-with-defaults {:a 1 :b 2} {:b 10 :c 20}) @{:a 1 :b 10 :c 20}))

(deftest defdoer
  (test-macro
    (defdoer :apk
      "manage APK packages"
      :name-is "package name")
    (upscope
      (def doer :apk)
      (def description "manage APK packages")
      (def mandatory-props-remove {})
      (def optional-props-ensure {})
      (def expand-resources {})
      (def mandatory-props-ensure {})
      (def name-is "package name")
      (def optional-props-remove {}))))

(deftest defhelper
  (test-macro
    (defhelper :smf :method
      "manage SMF methods"
      :timeout 50)
    (upscope
      (def doer :smf)
      (def description-method "manage SMF methods")
      (def timeout-method 50)
      (def mandatory-props-method {})
      (def optional-props-method {})))

  (test-macro
    (defhelper :zone :bootstrap
      "bootstrap a zone"
      :exactly-one-of [:server :file])
    (upscope
      (def doer :zone)
      (def description-bootstrap "bootstrap a zone")
      (def exactly-one-of-bootstrap [:server :file])
      (def mandatory-props-bootstrap {})
      (def optional-props-bootstrap {}))))

(deftest make-helper-fn
  (test-macro
    (make-helper-fn mirror)
    (defn mirror
      [name & spec]
      nil
      (pinpoint-error :mirror (when-defined exactly-one-of-mirror (if (not (has-exactly-one-of? exactly-one-of-mirror spec)) (errorf "need exactly one of %s" (comma-sep exactly-one-of-mirror)))) (when-defined at-least-one-of-mirror (if (not (has-at-least-one-of? at-least-one-of-mirror spec)) (errorf "need at-least one of %s" (comma-sep at-least-one-of-mirror)))) (let [user-spec (make-spec-struct :name name (splice spec)) expanded-spec (spec-with-defaults (extract-defaults mandatory-props-mirror) user-spec) spec-struct (checked-spec expanded-spec mandatory-props-mirror optional-props-mirror)] (struct :mirror spec-struct)))))

  (test-macro
    (make-helper-fn byhve :without-name true)
    (defn byhve
      [& spec]
      (def name "NO-NAME")
      (pinpoint-error :byhve (when-defined exactly-one-of-byhve (if (not (has-exactly-one-of? exactly-one-of-byhve spec)) (errorf "need exactly one of %s" (comma-sep exactly-one-of-byhve)))) (when-defined at-least-one-of-byhve (if (not (has-at-least-one-of? at-least-one-of-byhve spec)) (errorf "need at-least one of %s" (comma-sep at-least-one-of-byhve)))) (let [user-spec (make-spec-struct (splice spec)) expanded-spec (spec-with-defaults (extract-defaults mandatory-props-byhve) user-spec) spec-struct (checked-spec expanded-spec mandatory-props-byhve optional-props-byhve)] (struct :byhve spec-struct))))))

(deftest expand-list-struct
  (test
    (expand-list-struct {"key1" "val1" "key2" "val2"})
    @{"key1" "val1" "key2" "val2"})

  (test
    (expand-list-struct {["key1a" "key1b" "key1c"] "val1" "key2" "val2"})
    @{"key1a" "val1"
      "key1b" "val1"
      "key1c" "val1"
      "key2" "val2"}))

(deftest keys-in-struct
  (test (keys-in-struct [:a :b :c] {:a 1 :A 2 :alpha 3}) @[:a])
  (test (keys-in-struct [:a :b :c] {:a 1 :b 2}) @[:b :a])
  (test (keys-in-struct [:x :y :z] {:a 1 :b 2 :c 3}) @[]))

(deftest has-exactly-one-of?
  (test (has-exactly-one-of? [:a :b :c] [:a 1 :A 2 :alpha 3]) true)
  (test (has-exactly-one-of? [:a :b :c] [:a 1 :b 2 :c 3]) false)
  (test (has-exactly-one-of? [:x :y :z] [:a 1 :b 2 :c 3]) false))

(deftest has-none-or-one-of?
  (test (has-none-or-one-of? [:a :b :c] [:a 1 :A 2 :alpha 3]) true)
  (test (has-none-or-one-of? [:a :b :c] [:a 1 :b 2 :c 3]) false)
  (test (has-none-or-one-of? [:x :y :z] [:a 1 :b 2 :c 3]) true))

(deftest key-has-value?
  (test (key-has-value? [:a 1 :b 2] :a [1 2 3 4]) true)

  (test-error (key-has-value?
                [:a 1 :b 2] :a [3 4])
              "bad slot #3, expected string, symbol, keyword or buffer, got 1")

  (test-error (key-has-value?
                [:a 1 :b 2] :c [3 4])
              "key-has-value? did not find key :c"))

(deftest expand-list-struct
  (test (expand-list-struct {:a 1 :b 2}) @{:a 1 :b 2})
  (test (expand-list-struct {:a 1 [:x :y :z] 4}) @{:a 1 :x 4 :y 4 :z 4}))

(deftest safe-val
  (test (safe-val "word") "word")
  (test (safe-val "two words") "\"two words\"")
  (test (safe-val `"Quote this!", I said`) "\"\\\"Quote this!\\\", I said\""))
  )
