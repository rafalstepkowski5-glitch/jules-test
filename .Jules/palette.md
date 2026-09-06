## 2024-09-06 - Form Wrapping for Accessibility and Native Behaviors
**Learning:** In this application's custom UI, binding interactive elements to `click` handlers on buttons without wrapping them in semantic `<form>` tags prevents native keyboard accessibility (e.g., submitting via Enter key) and creates screen reader blind spots.
**Action:** Always wrap interactive inputs (like login credentials) in a `<form>` element, use `<button type="submit">`, and listen for the form's `submit` event with `event.preventDefault()` instead of raw button clicks.
