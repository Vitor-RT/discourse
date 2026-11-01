$LOAD_PATH.unshift(File.expand_path('../../crates/discourse_core/target/release', __dir__))
require 'discourse_core'

Rails.application.config.after_initialize do
  DiscourseCore.init_security
end
