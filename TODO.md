Make it so that when the first user is created - make him an admin and inform him about it. Don't require e-mail confirmation for this user.

_4_draft sequence diagram
_4_final
_5_draft
_5_final

# jak bd czas:
sequence diagrams - numeracja requestów
account activation seqence diagram - confirmation mail sent to client should be marked with a cloud.


Move from using a .env file to a config.toml.
Create logical sections in the config.toml. (like database, server, mail)
Use the config-rs crate to handle config.toml parsing.
Move the max_body_size argument to config.toml.
The config.toml is to be read during runtime, not compile-time.
Move the filenames of the database (prod and dev) to config.toml
Create an UPLOADS_FOLDER variable in config.toml.
Move to config.toml:
  - KEYRING_SERVICE_NAME
  - KEYRING_USERNAME
  - DB_KEY_ENV_VAR (add _NAME)
  - CLEANUP_INTERVAL_SECONDS

Move away from using clap and cli argument parsing in favor of config.toml.

Add ssl cert handling. (ca.crt, privkey.pem) (use use axum_server::tls_rustls::RustlsConfig; to implement it in the backend)
Configure HSTS.
Configure CORS securely.
Don't hardcode localhost in the frontend - just make requests to the same site it's served from.


Implement the Change password functionality with an e-mail confirmation.
Add security logging to the database for important actions like banning users, posts, login, register, sent emails, password change requests, etc. - Maybe a second DB would be better here?
Add soft delete for users.
Use translations in the frontend wherever possible.

