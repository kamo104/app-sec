Make it so that when the first user is created - make him an admin and inform him about it. Don't require e-mail confirmation for this user.

_4_draft sequence diagrams
_5_draft 

sequence diagrams - numeracja requestów
account activation seqence diagram - confirmation mail sent to client should be marked with a cloud.


To address these issues do the following:
  - move from using a .env file to a config.toml
  - create logical sections in the config.toml (like database, server, mail)
  - use the config-rs crate to handle config.toml parsing
  - move the max_body_size argument to config.toml.
  - the config.toml is to be read during runtime, not compile-time
  - move the names of the database (prod and dev) to config.toml
  - move to config.toml:
    - KEYRING_SERVICE_NAME
    - KEYRING_USERNAME
    - DB_KEY_ENV_VAR (add _NAME)
    - CLEANUP_INTERVAL_SECONDS
  - add ssl cert handling (ca.crt, privkey.pem) (use use axum_server::tls_rustls::RustlsConfig; to implement it in the backend)
  - configure HSTS
  - don't hardcode localhost in the frontend - just make requests to the same site it's served from.


Implement the Change password functionality with an e-mail confirmation.
Add security logging to the database for important actions like banning users, posts, login, register, sent emails, password change requests, etc.
Add soft delete for users.

