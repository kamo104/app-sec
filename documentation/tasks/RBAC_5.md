# Custom Web Application Development – Laboratory Instructions

**Poznan University of Technology**  
Faculty of Computing and Telecommunications  

**Course:** Application Security – Laboratories  
**Lecturer:** Michał Apolinarski, Ph.D.  
**Contact:** michal.apolinarski[at]put.poznan.pl  

**Topic:** Custom Web Application Development  
**Duration (on site):** 240 minutes  

---

## Prerequisites

- Completion of previous laboratories covering:
  - User registration
  - Login
  - Session management
  - Password reset
- Basic knowledge of:
  - Web applications
  - Databases
  - HTTP
  - Common web security issues

---

## Goals

The goal of this laboratory is to extend an existing **authentication system** into a small **content-based web service** (e.g. *Meme Service*, microblog, or similar idea) and apply **practical application security mechanisms** in a realistic scenario.

Students will design and implement:

- Role-based access control (RBAC)
- Secure handling of user-generated content:
  - Posts
  - Comments
  - Search
- Safe handling of file uploads
- Protection against common:
  - Access control vulnerabilities
  - Injection vulnerabilities
- Documentation of design and security decisions

---

## System Actors and Permissions

The system is based on distinct types of actors with different privileges and responsibilities:

### Guest Users (Unauthenticated)
- Browse public content
- Use keyword-based search

### Registered Users (Authenticated)
- Add new content (including file uploads)
- Comment on and rate posts
- Delete **only their own** content

### Administrators (Authenticated)
- Moderate the platform
- Delete or manage:
  - Any content item
  - Comments
  - User accounts

---

## Optional Component Features (for Extra Grade)

- CSRF protection
- Rate limiting for:
  - Comments
  - Ratings
  - Uploads
  - Search
- Content reporting and moderation queue (admin approval)
- Security event or audit logging
- Advanced file upload hardening:
  - Image re-encoding
  - Metadata stripping
- Security headers:
  - Content Security Policy (CSP)
  - X-Frame-Options
  - HSTS (if HTTPS is enabled)
- Soft-delete and restore functionality for administrators
- Optional features from previous laboratories
- Any other security-related feature proposed by the student

---

## Instructions (Tasks for a Group of Max. 2 Persons)

### Part A – Design (Draft Documentation)

1. Prepare **draft documentation** for a content platform integrated with your authentication module.  
   The document must include:
   - Full details of the student group, course, and exercise
   - Short description of the service and its actors
   - Functional and non-functional requirements (including security requirements)
   - Component architecture:
     - Simple diagram
     - Technology stack
     - Storage
   - Database structure:
     - Tables
     - Relations
     - Constraints
     - Triggers
   - UML sequence diagrams:
     - At least one new major process
     - Including alternative paths

2. Send your **draft documentation** to the lecturer for review.
3. Present and discuss your documentation with the lecturer.

> **Note:**  
> Include suffix `_draft` in the report filename.

---

### Part B – Implementation (Final Documentation)

1. After receiving feedback, implement the required functionality and update your documentation.
2. Prepare and send to the lecturer the **improved, final documentation**, including:
   - Screenshots
   - Explanations of key implementation choices
   - Description of security mechanisms
   - Conclusions
3. Demonstrate the working application.

> **Note:**  
> Include suffix `_final` in the report filename.

---

## Report Requirements

- Include a **title page** with full details of:
  - Student group
  - Course
  - Exercise
- The report should:
  - Be carefully edited
  - Provide evidence of completion of all exercises
  - Include screenshots, answers, and conclusions
- A **complete report** must be submitted to the lecturer **at least two days before** the next class in which it will be presented.

---

