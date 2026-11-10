(use judge)
(use ../src/gurp)
(import ../src/command-lib :only [strip-ansi])

(defn matches [haystack & needles]
  (count |(string/find $ haystack) (tuple ;needles)))

(deftest list-doers
  (test
    (matches (list-doers)
             "Install and uninstall APK"
             "Create and destroy zones")
    2))

(deftest help-for-doer
  (test
    (matches (command-lib/strip-ansi (help-for "bridge"))
             "Create and modify ethernet bridges."
             "Mandatory Properties"
             "Optional Properties"
             "(bridge/remove \"unwanted\")")
    4))

(deftest help-for-helper
  (test
    (matches (command-lib/strip-ansi (help-for "zone/network"))
             "Describe network configuration of a zone resource."
             "Mandatory Properties"
             "Optional Properties"
             "allowed-address [:string]")
    4))

(deftest control-data
  (test
    (matches (help-for "control-data")
             "The control-data function is used to set predefined parameters"
             "strict-hostname"
             "Send OpenTelemetry logs to the given endpoint")
    2))

(deftest doer/ensure
  (test
    (matches (help-for "file/ensure") "Create files from multiple sources")
    1))

(deftest doer/remove
  (test
    (matches (help-for "file/remove") "Create files from multiple sources")
    1))

(deftest nonsense
  (test (help-for "nonsense") "No help for 'nonsense'"))
