When /^I visit the homepage$/ do
  visit("/")
end

When /^I visit the signup page$/ do
  visit("/auth/sign_up")
end

When /^I visit the user profile page for "([^"]+)"$/ do |username|
  visit("/users/#{username}")
end

When /^I visit the settings page$/ do
  visit("/settings/profile")
end

When /^I visit the signin page$/ do
  visit("/auth/sign_in")
end

When /^I logout$/ do
  step %Q{I visit the homepage}
  click_link_or_button("sign out.")
end