# frozen_string_literal: true
begin
  require "discourse_core"
rescue LoadError
  begin
    require_relative "../../crates/discourse_core/target/debug/libdiscourse_core.so"
  rescue LoadError
  end
end
