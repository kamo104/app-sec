# Authentication Components

This directory contains reusable Vue components for authentication forms (login and registration).

## Component Overview

### AuthFormLayout
**Purpose**: Provides the consistent card layout and container structure for auth forms.
- **Props**: `title` (string) - The title displayed on the card
- **Features**:
  - Consistent styling and spacing
  - Form validation integration
  - Responsive layout

### UsernameField
**Purpose**: Reusable username input field with validation.
- **Props**:
  - `modelValue` (string) - Two-way bound username value
  - `required` (boolean, default: true)
  - `minLength` (number, default: 3)
  - `maxLength` (number, default: 20)
  - `allowPattern` (RegExp, default: `/^[a-zA-Z0-9_]+$/`)
- **Features**:
  - Real-time validation
  - Error state management
  - Touch tracking
  - Consistent styling

### EmailField
**Purpose**: Reusable email input field with validation.
- **Props**:
  - `modelValue` (string) - Two-way bound email value
  - `required` (boolean, default: true)
- **Features**:
  - Email format validation
  - Error state management
  - Touch tracking

### PasswordField
**Purpose**: Reusable password input field with WebAssembly-based strength validation.
- **Props**:
  - `modelValue` (string) - Two-way bound password value
  - `showStrength` (boolean, default: false) - Show password strength indicator
  - `required` (boolean, default: true)
- **Features**:
  - Show/hide password toggle
  - WebAssembly-based password validation (all validation rules enforced)
  - Optional strength indicator (0-7 score)
  - Error state management
  - Touch tracking
  - Real-time validation with detailed error messages

### ConfirmPasswordField
**Purpose**: Reusable confirm password field that validates against another password.
- **Props**:
  - `modelValue` (string) - Two-way bound confirm password value
  - `passwordToMatch` (string) - The password to validate against
  - `required` (boolean, default: true)
- **Features**:
  - Password matching validation
  - Show/hide toggle
  - Error state management
  - Touch tracking

### AuthSubmitButton
**Purpose**: Reusable submit button for auth forms.
- **Props**:
  - `label` (string) - Button text
  - `loading` (boolean, default: false)
  - `disabled` (boolean, default: false)
- **Features**:
  - Loading state
  - Disabled state
  - Consistent styling

### StatusMessage
**Purpose**: Reusable alert component for form status messages.
- **Props**:
  - `message` (string) - The message to display
  - `type` ('success' | 'error' | 'warning' | 'info', default: 'success')
- **Features**:
  - Closable
  - Type-based styling
  - Consistent appearance

### RememberMeCheckbox
**Purpose**: Reusable "Remember me" checkbox.
- **Props**:
  - `modelValue` (boolean) - Two-way bound checked state
- **Features**:
  - Consistent styling
  - Compact density

### ForgotPasswordLink
**Purpose**: Reusable "Forgot your password?" link button.
- **Props**:
  - `label` (string, default: "Forgot your password?")
- **Features**:
  - Text-only button styling
  - Info color

## Usage Examples

### Registration Form
```vue
<template>
  <AuthFormLayout title="User Registration">
    <template #default="{ handleSubmit: formSubmit }">
      <v-form @submit.prevent="handleSubmit" ref="form">
        <UsernameField v-model="formData.username" @touched="markFieldTouched('username')" />
        <EmailField v-model="formData.email" @touched="markFieldTouched('email')" />
        <PasswordField
          v-model="formData.password"
          show-strength
          @touched="markFieldTouched('password')"
          @validation="handlePasswordValidation"
        />
        <ConfirmPasswordField
          v-model="formData.confirmPassword"
          :password-to-match="formData.password"
          @touched="markFieldTouched('confirmPassword')"
        />
        <StatusMessage v-if="statusMessage" :message="statusMessage" :type="messageType" @close="clearMessage" />
        <AuthSubmitButton label="Register" :loading="loading" @click="() => formSubmit(handleSubmit)" />
      </v-form>
    </template>
  </AuthFormLayout>
</template>
```

### Login Form
```vue
<template>
  <AuthFormLayout title="User Login">
    <template #default="{ handleSubmit: formSubmit }">
      <v-form @submit.prevent="handleSubmit" ref="form">
        <UsernameField v-model="formData.username" @touched="markFieldTouched('username')" />
        <PasswordField
          v-model="formData.password"
          :show-strength="false"
          @touched="markFieldTouched('password')"
        />
        <RememberMeCheckbox v-model="formData.rememberMe" />
        <StatusMessage v-if="statusMessage" :message="statusMessage" :type="messageType" @close="clearMessage" />
        <AuthSubmitButton label="Login" :loading="loading" @click="() => formSubmit(handleSubmit)" />
        <ForgotPasswordLink @click="handleResetPassword" />
      </v-form>
    </template>
  </AuthFormLayout>
</template>
```

## Benefits

1. **Consistency**: All auth forms share the same styling and behavior
2. **Maintainability**: Changes to validation or styling are made in one place
3. **Reusability**: Components can be used across different auth flows
4. **Type Safety**: Full TypeScript support with proper interfaces
5. **Flexibility**: Props allow customization for different use cases
6. **Validation**: Built-in validation logic with error handling
7. **Accessibility**: Consistent ARIA labels and keyboard navigation

## Component Communication

- **v-model**: Two-way data binding for form values
- **@touched**: Emitted when a field is first interacted with
- **@validation**: Emitted with validation results (for complex fields like password)
- **@close**: Emitted when status message is closed
- **@click**: Emitted when buttons/links are clicked

## Validation Strategy

### WebAssembly Password Validation
All password validation is performed using WebAssembly for consistent validation rules between frontend and backend:
- **Real-time validation**: Errors are displayed as the user types
- **Detailed error messages**: Multiple validation rules with specific error messages
- **Strength scoring**: 0-7 score with visual indicator
- **Consistent rules**: Same validation logic as backend

### Field Validation
Each field component manages its own validation state and exposes a `validate()` method that returns:
```typescript
{
  valid: boolean,
  errors: string[]
}
```

The parent form component calls these methods on submit and handles the overall form validation.