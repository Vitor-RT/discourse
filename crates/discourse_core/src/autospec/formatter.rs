<<<<<<< HEAD
use std::fs::File;
=======
use std::fs::{self, File};
>>>>>>> origin/rust-migration-7847495834899956704
use std::io::Write;
use std::path::Path;

pub struct Formatter;

impl Formatter {
    const RUBY_FORMATTER_CODE: &'static str = r#"# frozen_string_literal: true

require "rspec/core/formatters/base_text_formatter"
require "parallel_tests/rspec/logger_base"

module Autospec
end

class Autospec::Formatter < RSpec::Core::Formatters::BaseTextFormatter
  RSpec::Core::Formatters.register self,
                                   :example_passed,
                                   :example_pending,
                                   :example_failed,
                                   :start_dump

  RSPEC_RESULT = "./tmp/rspec_result"

  def initialize(output)
    super
    FileUtils.mkdir_p("tmp") unless Dir.exist?("tmp")
    File.delete(RSPEC_RESULT) if File.exist?(RSPEC_RESULT)
    @fail_file = File.open(RSPEC_RESULT, "w")
  end

  def example_passed(_notification)
    output.print RSpec::Core::Formatters::ConsoleCodes.wrap(".", :success)
  end

  def example_pending(_notification)
    output.print RSpec::Core::Formatters::ConsoleCodes.wrap("*", :pending)
  end

  def example_failed(notification)
    output.print RSpec::Core::Formatters::ConsoleCodes.wrap("F", :failure)
    @fail_file.puts(notification.example.location + " ")
    @fail_file.flush
  end

  def start_dump(notification)
    output.puts
  end

  def close(filename)
    @fail_file.close
    super(filename)
  end
end
"#;

    pub fn setup_ruby_formatter() -> String {
        let path = "/tmp/discourse_autospec_formatter.rb";
        if !Path::new(path).exists() {
            let mut file = File::create(path).unwrap();
<<<<<<< HEAD
            file.write_all(Self::RUBY_FORMATTER_CODE.as_bytes())
                .unwrap();
=======
            file.write_all(Self::RUBY_FORMATTER_CODE.as_bytes()).unwrap();
>>>>>>> origin/rust-migration-7847495834899956704
        }
        path.to_string()
    }
}
