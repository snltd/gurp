// Compiles a Janet image file of all the code necessary to generate Markdown docs from doer
// definitions and examples, then runs it. The step of building the image lets us structure Janet
// in separate files with uses and includes, and also have a native CLI component.

fn main() {
    build_helper::ImageHelper::new(
        vec!["doers.janet", "doer-docs/markdown-docs.janet"],
        "markdown-docs.jimage",
    )
    .compile_to_file()
    .call_with_image("(generateall-docs)");
}
