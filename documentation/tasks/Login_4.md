# User Login, Session Management and Password Reset – Laboratory Instructions

**Poznan University of Technology**  
Faculty of Computing and Telecommunications  

**Course:** Application Security – Laboratories  
**Lecturer:** Michał Apolinarski, Ph.D.  
**Contact:** michal.apolinarski[at]put.poznan.pl  

**Topic:** User Login, Session Management and Password Reset Process  
**Duration (on site):** 240 minutes  
**Participants:** Groups of max. 2 persons  

---

## Prerequisites

- General knowledge of:
  - Computer networks
  - Operating systems
  - Databases
- Basic programming skills in any language
- Familiarity with:
  - Forms
  - Hashing
  - Tokens
  - Database design
  - UML modeling
- Completed **previous laboratory** with a working **registration module**

Reference:  
- https://www.visual-paradigm.com/guide/

---

## Goals

The purpose of this laboratory is to design and implement **secure mechanisms** for:

- User login and logout (session destruction)
- Server-side session management **or** token-based session management
- Password reset (“forgotten password”) feature

### Optional Component Features (for extra grade)

- Password strength meter / advanced password policy
- CSRF protection for all forms
- Rate limiting or account lockout mechanisms, CAPTCHA
- Device / session management (view and revoke active sessions)
- Security event logging (failed logins, invalid tokens, lockouts)
- Multi-factor authentication (MFA)
- Enhanced transport security:
  - Enforcing HTTPS
  - Secure cookies
  - HSTS  
  *(self-signed certificates acceptable)*
- Any additional security-related ideas

---

## Instructions (Tasks for a Group of Max. 2 Persons)

### Part A – Design

1. Using your existing project from the **registration laboratory**, extend the documentation to cover:
   - Login
   - Session management
   - Password reset feature

   The document must include:
   - Full details of the student group, course, and exercise
   - Updated short description of the complete authentication module, including:
     - Security assumptions
   - Updated **functional and non-functional requirements** for the new features
   - Updated database structure
   - UML sequence diagrams (at least one) for:
     - Login
     - Logout
     - Password reset request
     - Password reset completion  
     *(including validations and alternative paths)*

2. Send the **updated draft documentation** to the lecturer for review.
3. Present and discuss your documentation with the lecturer.

> **Notes:**
> - For diagrams, it is recommended to use **Draw.io**: https://app.diagrams.net  
> - All communication involving credentials (passwords, tokens) must be protected using **HTTPS** in production.
> - HTTPS configuration is not strictly required in the laboratory environment, but the design and documentation must clearly assume HTTPS usage.
> - Include suffix `_draft` in the report filename.

---

### Part B – Implementation

1. Extend your existing application to implement:
   - Login
   - Session management
   - Password reset process  
   according to your design.

2. Prepare and send to the lecturer the **improved, final documentation**, including:
   - Screenshots
   - Explanations of key implementation choices
   - Description of security mechanisms
   - Conclusions

3. Demonstrate the working functionality:
   - Show a complete authentication module
   - Explain your security-related decisions

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

