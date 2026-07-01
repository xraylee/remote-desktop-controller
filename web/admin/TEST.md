# RDCS Web Admin Console - Test Plan

**Version**: 1.0  
**Created**: 2026-06-29  
**Status**: Active  
**Framework**: Superpowers Testing Standards

---

## Overview

This document defines the comprehensive testing strategy for the RDCS Web Admin Console, a React + TypeScript management dashboard following the **Superpowers** testing framework standards.

### What We're Testing

The RDCS Web Admin Console is a single-page application (SPA) built with:
- **Frontend**: React 18 + TypeScript + Vite
- **Routing**: React Router v6
- **State Management**: Zustand (auth) + TanStack Query (server state)
- **Styling**: Tailwind CSS
- **HTTP Client**: Axios with interceptors

**Key Features**:
- User authentication (email/password + optional TOTP)
- Real-time dashboard with live statistics
- Device management
- Session monitoring
- Connection records audit
- Member management
- System settings

---

## Current State Assessment

### ⚠️ Critical Finding: No Test Infrastructure

**Status**: The project currently has **ZERO test coverage**.

| Component | Status | Evidence |
|-----------|--------|----------|
| Test Framework | ❌ Missing | No Jest/Vitest configured |
| Test Files | ❌ None | No `.test.ts` or `.spec.ts` files |
| Testing Library | ❌ Missing | No @testing-library/react |
| Test Scripts | ❌ Missing | No `test` command in package.json |
| E2E Tests | ❌ Missing | No Playwright/Cypress |
| Coverage Tools | ❌ Missing | No coverage configuration |

**Code Metrics**:
- Total lines: ~1,550 lines
- Files: 15 TypeScript/TSX files
- Components: 7 pages + 2 shared components
- Test coverage: **0%** ❌

---

## Test Strategy (Superpowers Framework)

### Test Pyramid

```
       /\
      /  \     E2E Tests (5%)
     /────\    - Critical user flows
    /      \   - Playwright
   /        \  
  /──────────\ Integration Tests (15%)
 /            \ - Component integration
/──────────────\ - API mocking
                
                Unit Tests (80%)
                - Components
                - Utilities
                - Stores
```

### Testing Levels

| Level | Target Coverage | Tools | Priority |
|-------|----------------|-------|----------|
| **Unit Tests** | 80% | Vitest + Testing Library | 🔴 Critical |
| **Integration Tests** | Key flows | MSW + Testing Library | 🟡 High |
| **E2E Tests** | Critical paths | Playwright | 🟢 Medium |
| **Visual Regression** | UI consistency | Chromatic (future) | ⚪ Low |

---

## Recommended Testing Stack

### Core Tools

```json
{
  "devDependencies": {
    "vitest": "^2.0.0",
    "@testing-library/react": "^16.0.0",
    "@testing-library/jest-dom": "^6.0.0",
    "@testing-library/user-event": "^14.0.0",
    "jsdom": "^24.0.0",
    "msw": "^2.0.0",
    "playwright": "^1.45.0",
    "@vitest/coverage-v8": "^2.0.0"
  }
}
```

### Configuration Files Needed

1. `vitest.config.ts` - Test runner configuration
2. `vitest.setup.ts` - Global test setup
3. `playwright.config.ts` - E2E test configuration
4. `src/test/setup.ts` - Custom test utilities
5. `src/test/mocks/handlers.ts` - MSW request handlers

---

## Test Cases (Superpowers AC Format)

### 1. Unit Tests - Authentication Store

#### 1.1 Login Flow
**File**: `src/stores/__tests__/authStore.test.ts`

**AC**: Login with valid credentials stores tokens and user data
```typescript
Given: User provides email "admin@example.com" and password "password123"
When: login() is called
Then: 
  - accessToken is stored in state and localStorage
  - refreshToken is stored in localStorage
  - isAuthenticated becomes true
  - member object is populated
```

**AC**: Login with invalid credentials throws error
```typescript
Given: User provides invalid credentials
When: login() is called
Then:
  - Promise rejects with error
  - isAuthenticated remains false
  - Tokens remain null
```

#### 1.2 Logout Flow
**AC**: Logout clears all auth state
```typescript
Given: User is logged in
When: logout() is called
Then:
  - All tokens cleared from state and localStorage
  - isAuthenticated becomes false
  - member becomes null
  - Server logout endpoint called
```

#### 1.3 Session Restoration
**AC**: restoreSession() recovers auth state from localStorage
```typescript
Given: Valid refresh_token exists in localStorage
When: restoreSession() is called
Then:
  - isAuthenticated becomes true
  - Tokens restored from localStorage
```

---

### 2. Unit Tests - API Client

#### 2.1 Request Interceptor
**File**: `src/api/__tests__/client.test.ts`

**AC**: Requests include Authorization header when token exists
```typescript
Given: Access token is set
When: Any API request is made
Then: Request includes "Authorization: Bearer <token>" header
```

#### 2.2 Response Interceptor (401 Handling)
**AC**: 401 response redirects to login
```typescript
Given: API returns 401 Unauthorized
When: Response is received
Then:
  - Access token cleared
  - User redirected to /login
```

---

### 3. Integration Tests - Login Page

#### 3.1 Login Form Submission
**File**: `src/pages/__tests__/LoginPage.test.tsx`

**AC**: Successful login redirects to dashboard
```typescript
Given: User is on /login
When: User enters valid credentials and submits
Then:
  - API POST /auth/login called
  - On success, redirected to /dashboard
  - Loading state shown during request
```

**AC**: Failed login shows error message
```typescript
Given: User is on /login
When: API returns 401 error
Then:
  - Error message displayed
  - Form remains interactive
  - No redirect occurs
```

---

### 4. Integration Tests - Dashboard Page

#### 4.1 Dashboard Data Loading
**File**: `src/pages/__tests__/DashboardPage.test.tsx`

**AC**: Dashboard displays stats after loading
```typescript
Given: User is authenticated
When: Dashboard page mounts
Then:
  - API GET /api/dashboard/stats called
  - Loading skeleton shown initially
  - Stats cards display data after load
  - Auto-refresh every 5 seconds
```

**AC**: Dashboard handles API errors gracefully
```typescript
Given: API returns 500 error
When: Dashboard tries to fetch stats
Then:
  - Error state shown (not white screen)
  - Retry mechanism available
```

---

### 5. Integration Tests - Protected Routes

#### 5.1 Auth Guard
**File**: `src/components/__tests__/ProtectedRoute.test.tsx`

**AC**: Unauthenticated users redirected to login
```typescript
Given: User is NOT authenticated
When: User tries to access /dashboard
Then: Redirected to /login
```

**AC**: Authenticated users can access protected routes
```typescript
Given: User is authenticated
When: User navigates to /dashboard
Then: Page renders normally
```

---

### 6. E2E Tests - Critical User Flows

#### 6.1 Complete Login Flow
**File**: `e2e/auth.spec.ts`

**Scenario**:
1. Navigate to http://localhost:3000
2. Redirected to /login
3. Enter email: "admin@example.com"
4. Enter password: "password123"
5. Click "登录" button
6. Wait for redirect to /dashboard
7. Verify URL is /dashboard
8. Verify dashboard content visible

**Expected**: Full flow completes in < 3 seconds

#### 6.2 Device Management Flow
**File**: `e2e/devices.spec.ts`

**Scenario**:
1. Login as admin
2. Navigate to /devices
3. View device list
4. Click "刷新" button
5. Verify device list updates

**Expected**: Device list loads and updates successfully

---

## Acceptance Criteria (Production Readiness)

### Must Pass (Blocking) ✅

- [ ] Unit test coverage ≥ 70%
- [ ] All critical paths have integration tests
- [ ] CI/CD pipeline runs tests automatically
- [ ] No console errors in tests
- [ ] Authentication flow fully tested

### Should Pass (Non-Blocking) ⚠️

- [ ] Unit test coverage ≥ 80%
- [ ] E2E tests for top 3 user flows
- [ ] Visual regression tests
- [ ] Accessibility tests (WCAG 2.1 AA)
- [ ] Performance tests (Lighthouse CI)

### Could Have (Nice to Have) 💡

- [ ] Storybook with component tests
- [ ] Mutation testing
- [ ] Contract testing with backend API
- [ ] Load testing

---

## Test Execution

### Setup Test Infrastructure

```bash
# 1. Install dependencies
npm install --save-dev vitest @testing-library/react \
  @testing-library/jest-dom @testing-library/user-event \
  jsdom msw @vitest/coverage-v8

# 2. Install E2E tools
npm install --save-dev playwright @playwright/test

# 3. Initialize Playwright
npx playwright install
```

### Run Tests

```bash
# Unit + Integration tests
npm test

# Watch mode (during development)
npm test -- --watch

# Coverage report
npm test -- --coverage

# E2E tests
npm run test:e2e

# All tests
npm run test:all
```

---

## Implementation Roadmap

### Phase 1: Foundation (Week 1) 🔴 Critical

**Goal**: Establish basic test infrastructure

1. ✅ Install Vitest + Testing Library
2. ✅ Configure vitest.config.ts
3. ✅ Set up test utilities (render helpers, mock providers)
4. ✅ Write first test (authStore.test.ts)
5. ✅ Configure CI to run tests

**Deliverable**: Green CI pipeline with ≥1 passing test

---

### Phase 2: Core Coverage (Week 2-3) 🟡 High

**Goal**: Achieve 50%+ coverage on critical paths

1. ✅ Test all store logic (authStore)
2. ✅ Test API client (interceptors)
3. ✅ Test LoginPage (form submission, error handling)
4. ✅ Test ProtectedRoute (auth guard)
5. ✅ Test DashboardPage (data loading)

**Deliverable**: 50% unit test coverage

---

### Phase 3: Integration Tests (Week 4) 🟡 High

**Goal**: Test component interactions with mocked API

1. ✅ Set up MSW (Mock Service Worker)
2. ✅ Create mock API handlers
3. ✅ Test login → dashboard flow
4. ✅ Test device page data loading
5. ✅ Test error states

**Deliverable**: Key user flows covered by integration tests

---

### Phase 4: E2E Tests (Week 5) 🟢 Medium

**Goal**: Validate critical flows in real browser

1. ✅ Set up Playwright
2. ✅ Write auth flow E2E test
3. ✅ Write device management E2E test
4. ✅ Configure E2E tests in CI

**Deliverable**: 3 E2E tests covering critical paths

---

### Phase 5: Advanced Testing (Future) ⚪ Low

1. Visual regression testing (Chromatic)
2. Accessibility testing (axe-core)
3. Performance testing (Lighthouse CI)
4. Contract testing (Pact)

---

## Coverage Goals

| Phase | Target Coverage | Timeline |
|-------|----------------|----------|
| Phase 1 | 10% | Week 1 |
| Phase 2 | 50% | Week 2-3 |
| Phase 3 | 70% | Week 4 |
| Phase 4 | 75% | Week 5 |
| Production | 80%+ | Ongoing |

---

## Current Gaps & Risks

### Critical Gaps ⚠️

1. **No Test Infrastructure**: Zero tests means high regression risk
2. **No CI Integration**: No automated quality gate
3. **No Type Testing**: TypeScript helps but doesn't catch runtime issues
4. **No E2E Coverage**: User flows untested

### Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Regression bugs | High | Implement Phase 1-2 ASAP |
| Auth vulnerabilities | Critical | Priority test for auth flow |
| API changes break UI | High | Integration tests with MSW |
| Poor UX on errors | Medium | Test error states explicitly |

---

## Testing Best Practices (Superpowers)

### ✅ Do

1. **Test behavior, not implementation**
   ```typescript
   // ✅ Good: Test what user sees
   expect(screen.getByText('登录成功')).toBeInTheDocument()
   
   // ❌ Bad: Test internal state
   expect(component.state.isSubmitting).toBe(false)
   ```

2. **Use Testing Library queries properly**
   - Prefer `getByRole`, `getByLabelText` (accessibility-friendly)
   - Avoid `getByTestId` unless necessary

3. **Mock at the network boundary (MSW)**
   - Don't mock axios directly
   - Mock HTTP responses with MSW

4. **Write tests that fail for the right reasons**
   - If code breaks, test should fail
   - If code works, test should pass

### ❌ Don't

1. Don't test third-party libraries
2. Don't test implementation details
3. Don't write brittle selectors (`.css-class-xyz`)
4. Don't skip cleanup (Testing Library handles this)

---

## CI/CD Integration

### GitHub Actions Workflow (Recommended)

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - run: npm ci
      - run: npm run lint
      - run: npm test -- --coverage
      - run: npm run test:e2e
      
      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: ./coverage/coverage-final.json
```

---

## Maintenance

### When to Update Tests

- ✅ New feature added → Add tests first (TDD)
- ✅ Bug fixed → Add regression test
- ✅ Component refactored → Update tests if behavior changed
- ✅ API changed → Update mock handlers

### Test Review Checklist

- [ ] Does test have clear AC (Arrange-Act-Assert)?
- [ ] Is test name descriptive?
- [ ] Does test fail when it should?
- [ ] Is test isolated (no global state leaks)?
- [ ] Are mocks cleaned up properly?

---

## References

- [Superpowers Testing Framework](https://superpowers.dev/testing)
- [Testing Library Best Practices](https://testing-library.com/docs/react-testing-library/intro/)
- [Vitest Documentation](https://vitest.dev/)
- [MSW Documentation](https://mswjs.io/)
- [Playwright Documentation](https://playwright.dev/)

---

**Maintained by**: RDCS Team  
**Last Updated**: 2026-06-29  
**Status**: 🔴 Critical - Test infrastructure needed ASAP
