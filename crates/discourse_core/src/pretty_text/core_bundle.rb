PrecompiledBundle.new(
  dir: "tmp/pretty-text-processor",
  filename_prefix: "pretty-text",
  dependency_globs:
    %w[
      pnpm-lock.yaml
      frontend/pretty-text-processor/**/*.{js,mjs,cjs,json}
      frontend/pretty-text/addon/**/*.js
      frontend/discourse-markdown-it/src/**/*.js
    ] + DISCOURSE_MODULES.map { |m| "frontend/discourse/app/#{m}.js" },
) do
  Discourse::Utils.execute_command(
    "pnpm",
    "-C=frontend/pretty-text-processor",
    "node",
    "build.mjs",
    "--discourse-modules=#{DISCOURSE_MODULES.join(",")}",
    chdir: Rails.root.to_s,
  )
end
