(use ./definers)
(import ../collector)

(defdoer :gem
  "Install and uninstall Ruby gems."
  :name-is "Gem name"

  (ensure
    (property "gem-path" [:string] :optional
              "Path to gem executable other than /opt/ooce/bin/gem")
    (property "source" [:string] :optional
              "Source other than RubyGems. Can contain tokens and usernames")
    (property "version" [:string] :optional "Gem version"))

  (remove
    (property "gem-path" [:string] :optional
              "Path to gem executable other than /opt/ooce/bin/gem")
    (property "version" [:string] :optional
              "Gem version"))

  (notes
    "Tries to minimise the calls to `gem install` by grouping together
     installs with similar parameters"
    "Only version numbers are supported, so `latest` won't work."
    "`gem/remove` takes no options, so removes all versions of the given gem."))

(make-fn :ensure :gem)
(make-fn :remove :gem)
