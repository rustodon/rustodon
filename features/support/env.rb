require "capybara/cucumber"
require "selenium/webdriver"
require "minitest/spec"
require "uri"

Capybara.javascript_driver = :selenium_chrome_headless

Capybara.configure do |config|
  config.run_server = false
  config.default_driver = :selenium_chrome_headless
  config.app_host = ENV["CUCUMBER_HOST"] || "http://localhost:8000/"
  config.ignore_hidden_elements = false
end


class MinitestWorld
  include Minitest::Assertions
  attr_accessor :assertions

  def initialize
    self.assertions = 0
  end
end

World do
  MinitestWorld.new
end

Before do
  database_url = URI.parse(ENV["DATABASE_URL"])
  if database_url.scheme == "postgres"
    # We clear the database entirely between each scenario and remigrate
    # In order to do so, we have to drop all existing connections if Postgres is
    # currently in use.
    database = database_url.path[1..-1]
    drop_all_connections = <<-EOSQL
      SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity WHERE pg_stat_activity.datname = '#{database}' AND pid <> pg_backend_pid();
    EOSQL
    pid = spawn({
      "PGPASS" => database_url.password,
      "PGUSER" => database_url.user,
      "PGHOST" => database_url.host,
      "PGDATABASE" => database
    }, "psql", "-c", drop_all_connections, out: "/dev/null", err: "/dev/null")
    Process.wait(pid)
    status = $?
    if !status.success?
      STDERR.puts "Failed to drop all connections, psql exit status: #{status.to_i}"
      exit 1
    end
  end
  %x{diesel database reset}
  status = $?
  if !status.success?
    STDERR.puts "Failed to reset database, diesel database reset exit status: #{status.inspect}"
    exit 1
  end
end