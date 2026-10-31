(use judge)
(use ../../src/doers/definers)

(deftest spec-with-defaults
  (test
    (spec-with-defaults {:val1 123} {:val1 111 :val2 222})
    @{:val1 123 :val2 222})

  (test
    (spec-with-defaults {} {:val1 111 :val2 222})
    @{:val1 111 :val2 222})

  (test
    (spec-with-defaults {:val1 999 :val2 888} {:val1 111 :val2 222})
    @{:val1 999 :val2 888}))

(deftest defdoer
  (test-macro
    (defdoer :pkgin
      "Install and uninstall pkgin packages. Only valid in a pkgsrc zone."
      :name-is "Package name"
      :notes
      ["You specify pkgs by name, so `openssl` rather than `openssl-3.3.2`."])
    (upscope
      (def doer :pkgin)
      (def attrs (@build-doer "Install and uninstall pkgin packages. Only valid in a pkgsrc zone." :name-is "Package name" :notes ["You specify pkgs by name, so `openssl` rather than `openssl-3.3.2`."])))))

(deftest make-fn
  (test-macro
    (make-fn ensure :apk)
    (defn ensure
      [resource-name & spec]
      (pinpoint-error ensure (->> (splice spec) (enrich-and-validate resource-name (attrs ensure) :name resource-name) (spec->resource :apk resource-name) (collector/push :ensure :apk))))))

(deftest defdoer
  (test-macro
    (defdoer file
      "Create files from multiple sources, or remove them."
      :name-is "Fully qualified path to file"

      (ensure
        :exactly-one-of [:content :from :from-url :from-struct]
        :expand-resources {:checksum true}

        (property "mode" [:string] :mandatory "Permissions, octal" :default "0644")

        (property "owner" [:string] :mandatory
                  "The username or UID of the user who owns this file"
                  :default "root"))

      (defnotes "note 1" "second note"))
    (upscope
      (def doer :file)
      (def attrs (@build-doer "Create files from multiple sources, or remove them." :name-is "Fully qualified path to file" (ensure :exactly-one-of [:content :from :from-url :from-struct] :expand-resources {:checksum true} (property "mode" [:string] :mandatory "Permissions, octal" :default "0644") (property "owner" [:string] :mandatory "The username or UID of the user who owns this file" :default "root")) (defnotes "note 1" "second note"))))))

(deftest property
  (test
    (property "mode" [:string] :mandatory "Permissions, octal" :default "0644")
    {:name :mode
     :section :mandatory
     :spec @{:default "0644"
             :help "Permissions, octal"
             :types [:string]}}))

(deftest ensure
  (test-macro
    (ensure
      :exactly-one-of [:content :from :from-url :from-struct]
      :expand-resources {:checksum true}

      (property "mode" [:string] :mandatory "Permissions, octal" :default "0644")

      (property "owner" [:string] :mandatory
                "The username or UID of the user who owns this file"
                :default "root"))
    (ensure :exactly-one-of [:content :from :from-url :from-struct] :expand-resources {:checksum true} (property "mode" [:string] :mandatory "Permissions, octal" :default "0644") (property "owner" [:string] :mandatory "The username or UID of the user who owns this file" :default "root"))))

(deftest build-doer
  (test
    (build-doer "test doer"
                :name-is "whatever"
                (ensure
                  :exactly-one-of [:property-1 :property-2]
                  (property "property-1" [:string] :mandatory "You must have this")
                  (property "property-2" [:number] :optional "You might like this")))
    @{:description "test doer"
      :ensure @{:defaults @{}
                :exactly-one-of [:property-1 :property-2]
                :props @{:mandatory @{:name {:types [:string]}
                                      :property-1 @{:help "You must have this"
                                                    :types [:string]}}
                         :optional @{:property-2 @{:help "You might like this"
                                                   :types [:number]}}}}
      :name-is "whatever"
      :remove @{:props @{:mandatory @{:name {:types [:string]}}}}}))


(deftest defhelper
  (test-macro
    (defhelper :file :checksum
      "Define a checksum to be used when fetching a remote file"
      :exactly-one-of [:literal :from-url :with-suffix]

      :mandatory-props
      {:algorithm {:types [:string]
                   :help "Algorithm used to calculate checksum"
                   :values algorithms}}

      :optional-props
      {:literal {:types [:string]
                 :help "Literal checksum"}
       :from-url {:types [:string]
                  :help "Fetch checksum from this URL"}
       :with-suffix {:types [:string]
                     :help "When fetching a file from a URL, append this suffix to
                        fetch the checksum"}})
    (upscope
      (def doer :file)
      (def description-checksum "Define a checksum to be used when fetching a remote file")
      (def attrs (@build-helper "Define a checksum to be used when fetching a remote file" :exactly-one-of [:literal :from-url :with-suffix] :mandatory-props {:algorithm {:help "Algorithm used to calculate checksum" :types [:string] :values algorithms}} :optional-props {:from-url {:help "Fetch checksum from this URL" :types [:string]} :literal {:help "Literal checksum" :types [:string]} :with-suffix {:help "When fetching a file from a URL, append this suffix to                        fetch the checksum" :types [:string]}})))))

# (deftest safe-table-passthru
#   (test (safe-table :key1 "val1" :key2 "val2")))

(deftest safe-table-something-to-change
  (test
    (safe-table :key3 "val3" :key4 "val4"
                [:checksum {:from-helper true
                            :algorithm "sha256"
                            :literal "abc123"
                            :unique true}]
                [:fs {:from-helper true :special "/u01" :dir "dir1"}]
                [:fs {:from-helper true :special "/u02" :dir "dir2"}]
                [:fs {:from-helper true :special "/u03" :dir "dir3"}])
    @{:key3 "val3"
      :key4 "val4"
      [:checksum
       {:algorithm "sha256"
        :from-helper true
        :literal "abc123"
        :unique true}]
        [:fs
         {:dir "dir1"
          :from-helper true
          :special "/u01"}]
      [:fs
       {:dir "dir2"
        :from-helper true
        :special "/u02"}]
        [:fs
         {:dir "dir3"
          :from-helper true
          :special "/u03"}]}))


# (deftest mk-fn-helper
# (test-macro
# (make-fn :checksum :file)))

(deftest protocol-opts
(test-macro
  (protocol-opts)
))
