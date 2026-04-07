(use ../../src/dsl)

(file/ensure "/example/file/from-content"
             :owner "sys"
             :mode "0600"
             :content (indoc `
                          words
                           and
                          stuff`))
