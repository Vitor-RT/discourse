# frozen_string_literal: true

desc "Vendor the icon package's sprites from node_modules, keeping dropped icon names renderable"
task "svgicons:sync" do
  system("cargo run --release --bin font_awesome_sync --manifest-path crates/discourse_core/Cargo.toml -- --sync", exception: true)
end

desc "Fail if the vendored sprites don't match the icon package in node_modules"
task "svgicons:check" do
  system("cargo run --release --bin font_awesome_sync --manifest-path crates/discourse_core/Cargo.toml -- --check", exception: true)
end

desc "Install JS packages, then run svgicons:sync"
task "svgicons:update" do
  system("pnpm install", exception: true)
  Rake::Task["svgicons:sync"].invoke
end
