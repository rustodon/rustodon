Then /I should see "([^"]+)"/ do |content|
  assert_includes(page.body, content)
end
