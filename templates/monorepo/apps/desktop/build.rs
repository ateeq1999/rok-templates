//! Generates the route tree from `src/routes`.

fn main() {
    rok_ui_build::routes("src/routes")
        .generate()
        .expect("valid route files");
}
