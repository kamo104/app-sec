## High-Level Workflow

The process involves two main phases: **Generation** (when the user signs up) and **Processing** (when the user clicks the link in their email).

---

## 1. Generation Logic

When a user registers, the system should generate a secure, temporary "opaque" token.

* **Entropy:** Use a Cryptographically Secure Pseudo-Random Number Generator (CSPRNG). A 32-byte or 64-character hex/base64 string is standard.
* **Storage:** **Never store tokens in plain text.** Treat them like passwords. Store a **salted hash** (e.g., Argon2) of the token in your database.
* **Expiration (TTL):** Set an expiration window of **2 hours**.
* **Database Schema:**
| Column | Type | Description |
| :--- | :--- | :--- |
| `user_id` | UUID / INT | Foreign key to the Users table. |
| `token_hash` | STRING | The hashed version of the activation token. |
| `expires_at` | TIMESTAMP | When the token becomes invalid. |
| `created_at` | TIMESTAMP | Audit trail for generation time. |

---

## 2. Processing Logic

When the user hits the activation endpoint (e.g., `https://myapp.com/activate?token=abc123...`), follow these steps:

1. **Retrieve & Hash:** Take the raw token from the URL and hash it using the same algorithm used during generation.
2. **Lookup:** Find the record in your `tokens` table matching that hash.
3. **Validate:** * **Existence:** If no record is found, return a generic "Invalid or expired link" error.
* **Expiration:** Check if `current_time > expires_at`.
* **Status:** Ensure the user isn't already activated (to prevent unnecessary processing).


4. **Execute:** * Update the `users` table: set `is_active = true` or `email_verified_at = now()`.
* **Delete the token:** Immediately remove the token record from the database to ensure it is **single-use**.


5. **User Feedback:** Redirect the user to a "Success" page or directly to their dashboard with a welcome message.

---

## 3. Security Best Practices

* **Generic Error Messages:** Do not tell the user *why* a token failed (e.g., "User not found"). Use "Invalid or expired link" for all failure cases.
* **HTTPS Only:** Ensure the link in the email uses `https://` to prevent the token from being intercepted via man-in-the-middle attacks.
* **No PII in URLs:** Do not include the user's email or ID in the activation URL. The token itself should be the only identifier needed to find the record.
