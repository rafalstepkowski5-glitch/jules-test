## 2024-03-01 - Form Accessibility and Keyboard Navigation
**Learning:** Found that login inputs were missing proper form wrappers, labels, and relied on simple button click listeners. This broke native keyboard submission behaviors (like pressing Enter) and made the form difficult for screen readers to interpret.
**Action:** Always wrap interactive inputs in semantic `<form>` tags, add `<label>` associations for screen readers, and use `<button type="submit">` with a `submit` event listener (using `e.preventDefault()`) instead of a simple click listener.
