(use ./lib)
(import ../collector)

(defdoer :apk
  "Install and uninstall APK packages. Only valid in an Alpine LX zone."
  :name-is "Package name"
  :notes ["Only adds and removes packages. You cannot specify or pin package versions."
          "The package database is refreshed prior to an install"])

(defensure "apk")
(defremove "apk")
