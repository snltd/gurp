(use ./lib)
(import ../collector)

(defdoer :pkgin
  "Install and uninstall pkgin packages. Only valid in a pkgsrc zone."
  :name-is "Package name"
  :notes
  ["You specify pkgs by name, so `openssl` rather than `openssl-3.3.2`. This
    means you can't request specific versions."])

(defensure "pkgin")
(defremove "pkgin")
