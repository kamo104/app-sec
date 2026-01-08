# Project Context

When working with this codebase, prioritize readability over cleverness. Ask clarifying questions before making architectural changes. DON'T use any emojis in the printed text or comments. Remember to always test your changes with the build.sh script!

## About This Project

A web app demonstration focused on using good web security practices.

## Key Directories

- `frontend/` - an npm, vue.js + vuetify web frontend,
- `backend/` - a rust axum backend server that has ../frontend/dist linked to it so that it can serve the web files,
- `proto/` - protobuf api buffers,
- `api-translator/` - rust crate compiled natively to the backend as well as the frontend using wasm-pack that is mainly used for translating encoded responses or errors to text,
- `field-validator/` - rust crate compiled to backend and frontend that is responsible for validating all data.

## Standards

- Type hints required on all TS functions
