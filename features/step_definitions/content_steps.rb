Then /^I should see "([^"]+)"$/ do |content|
  assert_includes(page.body, content)
end

When /^I fill in the signup form with the username "([^"]+)", password "([^"]+)" and email "([^"]+)"$/ do |username, password, email|
  fill_in "username", with: username
  fill_in "email", with: email
  fill_in "password", with: password
end

When /^I fill in the login form with the username "([^"]+)" and password "([^"]+)"$/ do |username, password|
  fill_in "username", with: username
  fill_in "password", with: password
end

When /^I submit the signup form$/ do
  find("form#signup button[type='submit']").click
end

When /^I submit the login form$/ do
  find("form#signin button[type='submit']").click
end

When /^I login with username "([^"]+)" and password "([^"]+)"$/ do |username, password|
  step %Q{I visit the signin page}
  step %Q{I fill in the login form with the username "#{username}" and password "#{password}"}
  step %Q{I submit the login form}
end

Given /^I am the user "([^"]+)"$/ do |username|
  step %Q{the user "#{username}" exists}
  step %Q{I login with username "#{username}" and password "password"}
end

Given /^the user "([^"]+)" exists$/ do |username|
  step %Q{I visit the signup page}
  step %Q{I fill in the signup form with the username "#{username}", password "password" and email "#{username}@test.org"}
  step %Q{I submit the signup form}
end

When /^I fill in the biography form with "([^"]+)"/ do |bio|
  fill_in "summary", with: bio
end

When /^I submit the biography form$/ do
  click_link_or_button("update")
end

When /^I have the bio "([^"]+)"$/ do |bio|
  step %Q{I visit the settings page}
  step %Q{I fill in the biography form with "#{bio}"}
  step %Q{I submit the biography form}
end

Then /^I should see a link to the profile for "([^"]+)" in "([^"]+)"$/ do |user, selector|
  within(selector) do
    find("a[href*='#{user}']", text: "@#{user}")
  end
end

Then /^I should see a link to "([^"]+)" in "([^"]+)"$/ do |href, selector|
  within(selector) do
    find("a[href='#{href}']")
  end
end

Then /^I should( not)? see a "([^"]+)" tag in "([^"]+)"$/ do |check_absence, tagname, selector|
  check_absence = !!check_absence
  within(selector) do
    tag = all(tagname)
    if check_absence
      assert(tag.size == 0)
    else
      assert(tag.size > 0)
    end
  end
end