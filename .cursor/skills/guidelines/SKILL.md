---
name: test-driven-development
description: Follow test-driven development (TDD) workflow: write tests first, then implement code to make tests pass, refactor, and iterate until everything works.
---

# Test-Driven Development (TDD)

This skill guides you to follow a strict test-driven development workflow. Always write tests before implementing code, then develop the minimal code needed to make tests pass.

## When to Use

- Use this skill when implementing new features or functionality
- Use when fixing bugs (write a failing test first that reproduces the bug)
- Use when refactoring existing code (ensure tests exist first)
- Use for any code changes that require verification

## TDD Workflow

Follow the Red-Green-Refactor cycle:

### 1. Red: Write a Failing Test

**First, write a test that describes the desired behavior:**

- Write the test before any implementation code
- The test should fail initially (Red phase)
- Make the test specific and clear about what it's testing
- Include edge cases and error conditions
- For integration tests, test the full workflow
- For unit tests, test individual functions/methods in isolation

**Example approach:**
- Identify what needs to be implemented
- Write a test that calls the function/feature with expected inputs
- Assert the expected outputs or behaviors
- Run the test to confirm it fails (this validates the test is actually testing something)

### 2. Green: Implement Minimal Code to Pass

**Write the simplest code that makes the test pass:**

- Implement only what's necessary to make the test pass
- Don't over-engineer or add extra features yet
- Focus on making the test green (passing)
- If the test is complex, break it down into smaller tests

**Example approach:**
- Implement the function/feature with minimal logic
- Use simple, straightforward implementations
- Hardcode values if needed initially (we'll refactor later)
- Run tests to confirm they pass

### 3. Refactor: Improve Code Quality

**Once tests pass, improve the code without changing behavior:**

- Refactor for clarity, performance, or maintainability
- Ensure all tests still pass after refactoring
- Remove duplication
- Improve naming and structure
- Add documentation/comments if needed

### 4. Repeat: Continue the Cycle

**Continue the Red-Green-Refactor cycle for each new requirement:**

- Add more tests for additional functionality
- Implement code to pass new tests
- Refactor as needed
- Keep iterating until the feature is complete

## Implementation Guidelines

### Test Structure

- **Unit tests**: Test individual functions, methods, or modules in isolation
- **Integration tests**: Test how multiple components work together
- **Test organization**: Group related tests logically
- **Test naming**: Use descriptive names that explain what is being tested

### Code Implementation

- Start with the simplest implementation that works
- Don't add features not covered by tests
- Make tests pass, then improve
- Keep the test suite running frequently

### Error Handling

- Write tests for error cases first
- Test both expected errors and unexpected scenarios
- Ensure error messages are clear and helpful

### Iteration

- Work in small, incremental steps
- Run tests after each change
- If a test fails unexpectedly, fix it before moving on
- Don't skip the refactor step - it's important for code quality

## Best Practices

1. **One test at a time**: Focus on making one test pass before moving to the next
2. **Keep tests fast**: Tests should run quickly to enable rapid feedback
3. **Test behavior, not implementation**: Test what the code does, not how it does it
4. **Maintain test independence**: Tests should not depend on each other
5. **Clean up**: Remove any temporary code or debugging statements during refactoring
6. **Document as you go**: Add comments and documentation during the refactor phase

## Example Workflow

```
1. Write test for new feature → Test fails (Red)
2. Implement minimal code → Test passes (Green)
3. Refactor code → Tests still pass
4. Write test for edge case → Test fails (Red)
5. Update implementation → Test passes (Green)
6. Refactor → Tests still pass
7. Continue until feature is complete
```

Remember: The goal is to have working, tested code. Tests are your safety net and documentation of expected behavior.
