# Project Context

When working with this codebase, prioritize readability over cleverness. Ask clarifying questions before making architectural changes. Don't run the front-end development server by yourself - it's already running in the background.

## About This Project

A web app demonstration focused on using good web security practices.

## Key Directories

- `frontend/` - an npm, vue.js + vuetify web frontend
- `backend/` - a rust axum backend server that has ../frontend/dist linked to it so that it can serve the web files

## Standards

- Type hints required on all TS functions
