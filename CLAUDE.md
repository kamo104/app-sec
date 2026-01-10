# Project Context

When working with this codebase:
  - Ask clarifying questions before making architectural changes or changing anything with the dependency tree.
  - DON'T use any emojis in the printed text or comments.
  - Remember to always test your changes with the build.sh script.
  - DON'T repeat yourself.
  - DON'T repeat code across different modules - try to look if a funciton is already implemented somewhere else.
  - Write as little code as possible to achieve the stated goal.
  - Write as few adapters as possible and use the provided API or functions directly.
  - NEVER use hard coded text in the user facing text - use the translator.
  - NEVER use magic numbers - use a const variable.

## About This Project

A web app demonstration focused on using good web security practices.

## Key Directories

- `frontend/` - an npm, vue.js + vuetify web frontend,
- `backend/` - a rust axum backend server that has ../frontend/dist linked to it so that it can serve the web files,
- `translator/` - rust crate compiled natively to the backend as well as the frontend using wasm-pack that is used for translating user facing errors from codes to text,
- `field-validator/` - rust crate compiled to backend and frontend that is responsible for validating all data,
- `api-types/` - rust crate shared across the project for defining api response types as precisely as possible.

## Standards

- Type hints required on all TS functions
