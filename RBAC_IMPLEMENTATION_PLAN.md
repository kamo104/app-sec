# RBAC Implementation Plan (Task 5)

## Design Decisions
- **Content type**: Image memes with title/description
- **Rating system**: Thumbs up/down (stored as +1/-1, net score = SUM)
- **Soft delete**: Yes (admin restore feature via `deleted_at` column)
- **User Roles**: Guest (view/search), User (create/comment/rate/delete own), Admin (moderate all, manage users, restore deleted)

---

## Completed

### 1. Database Layer
- [x] `backend/src/db/posts.rs` - Posts table with soft delete, search, pagination
- [x] `backend/src/db/comments.rs` - Comments table with soft delete
- [x] `backend/src/db/ratings.rs` - Ratings table with upsert (one vote per user per post)
- [x] `backend/src/db/mod.rs` - Integrated new tables into `DBHandle`

### 2. API Types
- [x] `api-types/src/enums.rs` - Added `FieldType::PostTitle`, `PostDescription`, `CommentContent`
- [x] `api-types/src/requests.rs` - Added `CreatePostRequest`, `UpdatePostRequest`, `PaginationQuery`, `SearchQuery`, `CreateCommentRequest`, `RatePostRequest`, `UpdateUserRoleRequest`
- [x] `api-types/src/responses.rs` - Added `PostResponse`, `PostListResponse`, `CreatePostResponse`, `PostError`, `PostErrorResponse`, `CommentResponse`, `CommentListResponse`, `CreateCommentResponse`, `CommentError`, `CommentErrorResponse`, `RatingResponse`, `RatingError`, `RatingErrorResponse`, `UserInfoResponse`, `UserListResponse`, `DeletedPostResponse`, `DeletedPostsListResponse`
- [x] Added `UserRole` to `LoginResponse` and `AuthSessionResponse`

### 3. Field Validation
- [x] `field-validator/src/lib.rs` - Added `validate_post_title()`, `validate_post_description()`, `validate_comment_content()` functions

### 4. Auth Extractors
- [x] `backend/src/api/auth_extractor.rs` - Added `AdminUser` extractor (401 if not auth, 403 if not admin)

### 5. API Endpoints
- [x] `backend/src/api/posts.rs` - List, search, get, create (multipart), update, delete, **get_post_image (secure)**
- [x] `backend/src/api/comments.rs` - List, create, delete comments
- [x] `backend/src/api/ratings.rs` - Rate post, remove rating
- [x] `backend/src/api/admin.rs` - List users, update role, delete user, list deleted posts, restore post

### 6. Router & Security
- [x] `backend/src/main.rs` - Registered all new routes
- [x] Secure image serving (no static `/uploads`, images served via `/api/posts/{id}/image` with soft-delete check)
- [x] `mime_guess` crate for content type detection

### 7. Frontend - Auth Store
- [x] Store user role in auth store from login/session response
- [x] Added `isAdmin` computed property

### 8. Frontend - Posts Feature
- [x] `frontend/src/pages/index.vue` - Posts feed with pagination and search
- [x] `frontend/src/pages/posts/[id].vue` - Post detail with comments and ratings
- [x] `frontend/src/pages/posts/new.vue` - Create post form with image upload

### 9. Frontend - Admin Panel
- [x] `frontend/src/pages/admin/users.vue` - User management (list, change role, delete)
- [x] `frontend/src/pages/admin/deleted-posts.vue` - Deleted posts management (list, restore)

### 10. Frontend - Navigation & Guards
- [x] `frontend/src/layouts/default.vue` - Navigation bar with user menu and admin dropdown
- [x] `frontend/src/router/index.ts` - Route guards for auth and admin routes

---

## API Endpoints Reference

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/posts` | Optional | List posts with pagination |
| GET | `/api/posts/search` | Optional | Search posts |
| GET | `/api/posts/{id}` | Optional | Get single post |
| GET | `/api/posts/{id}/image` | None | Get post image (checks soft-delete) |
| POST | `/api/posts` | Required | Create post (multipart) |
| PUT | `/api/posts/{id}` | Required | Update post (owner/admin) |
| DELETE | `/api/posts/{id}` | Required | Soft delete post (owner/admin) |
| GET | `/api/posts/{id}/comments` | None | List comments |
| POST | `/api/posts/{id}/comments` | Required | Create comment |
| DELETE | `/api/comments/{id}` | Required | Delete comment (owner/admin) |
| POST | `/api/posts/{id}/rate` | Required | Rate post (+1/-1) |
| DELETE | `/api/posts/{id}/rate` | Required | Remove rating |
| GET | `/api/admin/users` | Admin | List all users |
| PUT | `/api/admin/users/{id}/role` | Admin | Update user role |
| DELETE | `/api/admin/users/{id}` | Admin | Delete user |
| GET | `/api/admin/posts/deleted` | Admin | List deleted posts |
| POST | `/api/admin/posts/{id}/restore` | Admin | Restore deleted post |

---

## Frontend Pages

| Path | Auth | Description |
|------|------|-------------|
| `/` | None | Posts feed with search and pagination |
| `/posts/:id` | None | Post detail with comments and ratings |
| `/posts/new` | Required | Create new post form |
| `/admin/users` | Admin | User management |
| `/admin/deleted-posts` | Admin | Deleted posts management |

---

## Security Features Implemented

1. **Role-Based Access Control**
   - Guest: View posts, search
   - User: Create posts/comments, rate, delete own content
   - Admin: All user permissions + moderate all content, manage users, restore deleted posts

2. **Secure Image Handling**
   - Images served through authenticated endpoint, not static files
   - Soft-deleted post images return 404
   - Image validation (magic bytes, size limits, allowed formats)
   - `mime_guess` for content type detection

3. **Route Guards**
   - Auth-required routes redirect to login
   - Admin-only routes redirect to home for non-admins

4. **Soft Delete**
   - Posts and comments use soft delete (`deleted_at` column)
   - Admins can restore deleted posts
