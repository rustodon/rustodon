Feature: Basic Functionality

  Scenario: Visit homepage
    When I visit the homepage
    Then I should see "Rustodon"

  Scenario: Signup
    When I visit the signup page
    And I fill in the signup form with the username "test", password "test" and email "test@test.org"
    And I submit the signup form
    Then I should see "signed up!"

  Scenario: My user page
    Given I am the user "test_user"
    And I visit the user profile page for "test_user"
    Then I should see "test_user"

  Scenario: Changing my bio
    Given I am the user "test_user"
    And I have the bio "awoo"
    And I visit the user profile page for "test_user"
    Then I should see "awoo"

  Scenario: Mentions in bio
    Given the user "other_user" exists
    And I am the user "test_user"
    And I have the bio "@other_user exists"
    And I visit the user profile page for "test_user"
    Then I should see a link to the profile for "other_user" in ".p-note"

  Scenario: Links in bio
    Given I am the user "test_user"
    And I have the bio "Signed up at http://glitch.social"
    And I visit the user profile page for "test_user"
    Then I should see a link to "http://glitch.social" in ".p-note"

  Scenario: HTML escaped in bio
    Given I am the user "test_user"
    And I have the bio "<script type='text/javascript'>alert('malicious');</script>"
    And I visit the user profile page for "test_user"
    Then I should not see a "script" tag in ".p-note"