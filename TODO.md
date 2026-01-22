prio::critical:
  - Add a default admin variables - username, email, password - which gets loaded as an admin account if there's no users in the database. Remove the current default behaviour of adding admin privilages to the first user that is created. Update the latex documentation to include this behaviour.
  - merge the base_url_dev and base_url_prod into a single variable base_url.
  - remove the dev_mode variable and change it to individual variables: encrypt_database and openapi (false/true)
  - Create a README.md that details how to host this app in the prod mode.

prio::high:
  - There are inaccurate warning messages - when an admin is trying to delete a user - nothing should be shown. When an admin is deleting a post - there should be a warning with an information that the post will be banned (not deleted). When a user is deleting a post - there should be a warning that the post is going to be deleted permamently.
  - Make sure there are no hard coded user facing strings on the frontend. They should all be available in the translation file.

prio::low:
  - Use translations in the frontend wherever possible.

prio::nice_to_have:
  - Add security logging to the database for important actions like banning users, posts, login, register, sent emails, password change requests, etc.


documentation:
  - _5_draft + sequence diagrams
  - _5_final

