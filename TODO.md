prio::critical:
  - Make it so that when the first user is created - make him an admin.
  - Don't hardcode the localhost endpoint in the frontend - just make requests to the same site it's served from.

prio::high:
  - Add ssl cert handling. (ca.crt, privkey.pem) (use use axum_server::tls_rustls::RustlsConfig; to implement it in the backend)
  - Configure HSTS.
  - Configure CORS securely.

prio::medium:
  - Add soft delete for users.

prio::low:
  - Implement the Change password functionality with an e-mail confirmation.
  - Add security logging to the database for important actions like banning users, posts, login, register, sent emails, password change requests, etc. - Maybe a second DB would be better here?
  - Use translations in the frontend wherever possible.



documentation:
  - _4_final screenshots
  - _5_draft sequence diagrams
  - _5_final

