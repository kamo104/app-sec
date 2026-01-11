# User Registration Process – Laboratory Instructions

**Poznan University of Technology**  
Faculty of Computing and Telecommunications  

**Course:** Application Security – Laboratories  
**Lecturer:** Michał Apolinarski, Ph.D.  
**Contact:** michal.apolinarski[at]put.poznan.pl  

**Topic:** User Registration Process  
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

Reference:  
- https://www.visual-paradigm.com/guide/

---

## Goals

The purpose of this laboratory is to design and implement a **secure user registration module**, including:

- A user registration form (optionally with local input validation)
- Server-side input validation and error handling
- Secure password handling
- Storing user data in a database
- Generating and processing account activation tokens
- Creating software documentation (UML and architecture)
- Presenting and demonstrating a working prototype

### Optional Registration Component Features (for extra grade)

- Password strength meter / advanced password policy
- Email verification via SMTP server / DNS MX lookup
- Domain restrictions for e-mail addresses (whitelist / blacklist)
- Invite tokens
- Rate limiting / CAPTCHA
- Improved password hashing
- Activation-token hardening
- Security event logging
- Enforcing HTTPS and basic transport security
- Any additional security-related ideas

> **Note:**  
> It is recommended to implement the module as a **web application**, but desktop or mobile applications are also acceptable.  
> The business domain of the application is irrelevant — focus on the **authentication component**, not the full application.

---

## Important Note

This laboratory forms the foundation for the next topics:

- **Login and session management**
- **Password reset feature**

---

## Instructions (Tasks for a Group of Max. 2 Persons)

### Part A – Design

1. Prepare **draft documentation** describing your planned registration module.  
   The document must include:
   - Full details of the student group, course, and exercise
   - A short description of the component:
     - Purpose
     - Data collected
     - Security assumptions
   - Component requirements:
     - Functional requirements
     - Non-functional requirements
   - Component architecture:
     - Simple diagrams
     - Technology stack
   - Database structure:
     - Tables
     - Fields
     - Constraints
   - UML sequence diagrams for:
     - Registration
     - Account activation  
     (including validations and alternative paths)

2. Send your **draft documentation** to the lecturer for review.
3. Present and discuss your documentation with the lecturer.

> **Notes:**
> - For diagrams, it is recommended to use **Draw.io**: https://app.diagrams.net  
> - All communication involving credentials (passwords, activation tokens) must be protected using **HTTPS** in production.  
> - HTTPS configuration is not strictly required in the laboratory environment, but the design must clearly assume HTTPS usage.
> - Include suffix `_draft` in the report filename.

---

### Part B – Implementation

1. Build the designed registration components.
2. Prepare and send to the lecturer the **improved, final documentation**, including:
   - Screenshots
   - Explanations of key implementation choices
   - Description of security mechanisms
   - Conclusions

3. Demonstrate the working functionality:
   - Show a complete registration and activation flow
   - Explain security-related decisions

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

