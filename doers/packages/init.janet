(use sh)
(use judge)
(use "../core/core")

(def my-name "packages")

(defn- run-pkg-cmd
  "Shells out to the pkg(5) command. This is expensive, so it's memoized"
  []
  (delay
    ($< /bin/pkg list -aH -o "name,flags")))

(def pkg-command (run-pkg-cmd))

(defn- parse-pkg-output
  "Turns a single string of output from pkg(5) into an array of alternating package names and flags"
  [pkg-output]
  (->>
    pkg-output
    (string/trim)
    (string/split "\n")
    (mapcat |(string/split " " $ 0 2))
    (map string/trim)))

(defn- installed-packages
  "Turns a parse-pkg-output array into a list of installed package names"
  [pkg-list]
  (->>
    pkg-list
    (partition 2)
    (filter |(string/has-prefix? "i" (1 $)))
    (map first)))

(defn- available-packages
  "Turns a parse-pkg-output array into a list of packages which are available, but not installed "
  [pkg-list]
  (->>
    pkg-list
    (partition 2)
    (filter |(not (string/has-prefix? "i" (1 $))))
    (map first)))

(defn- packages-to-add
  "Returns an array of the requested packages which can be installed"
  [requested-packages installed-packages available-packages]
  (->>
    requested-packages
    (filter |(not (has-value? installed-packages $)))
    (filter |(has-value? available-packages $))))

(defn- packages-to-remove
  "Returns an array of unwanted packges which are currently installed"
  [unwanted-packages installed-packages]
  (->>
    unwanted-packages
    (filter |(has-value? installed-packages $))))

(defn- install-packages
  [package-list]
    (if (empty? package-list)
      (say my-name "no packages to add")
      (enact-sh! (/bin/pkg install ;package-list))))

(defn- remove-packages
  [package-list]
    (if (empty? package-list)
      (say my-name "no packages to remove")
      (enact-sh! (/bin/pkg uninstall ;package-list))))
    

(defn add [& requested-packages]
  (install-packages
    (let [installed-packages (->>
                               (pkg-command)
                               (parse-pkg-output)
                               (installed-packages))
          available-packages (->>
                               (pkg-command)
                               (parse-pkg-output)
                               (available-packages))]
      (packages-to-add requested-packages installed-packages available-packages))))

(defn remove [& unwanted-packages]
  (remove-packages
    (->>
      (pkg-command)
      (parse-pkg-output)
      (installed-packages)
      (packages-to-remove unwanted-packages))))

(test (packages-to-add
        ["helix" "janet" "oozone"]
        ["helix" "rust" "zcage"]
        ["helix" "janet" "vim" "flac" "lame"])
      @["janet"])

(test (packages-to-remove
        ["go" "perl" "python"]
        ["go" "rust" "clojure" "ruby" "elixir"])
      @["go"])

(deftest test-package-lists
  (let [pkg-list (parse-pkg-output (slurp "sample"))]
    (test (length (installed-packages pkg-list)) 613)
    (test (length (available-packages pkg-list)) 521)))
