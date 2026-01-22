The paths are still being duplicated only now in the ApiDoc derive and in the router. The routes can all be declared with a comma as well like so: 'OpenApiRouter::new().routes(routes!(get_user, post_user));'. Is there any way of declaring the api functions in a single place?

prio::critical:
  - Update the documentation/latex/10-usage/main.tex to match the new code.

prio::high:
  - There are inaccurate warning messages - when an admin is trying to delete a user - nothing should be shown. When an admin is deleting a post - there should be a warning with an information that the post will be banned (not deleted). When a user is deleting a post - there should be a warning that the post is going to be deleted permamently.
  - Make sure there are no hard coded user facing strings on the frontend. They should all be available in the translation file.

prio::nice_to_have:
  - Add security logging to the database for important actions like banning users, posts, login, register, sent emails, password change requests, etc.


documentation:
  - _5_draft + sequence diagrams
  - _5_final

