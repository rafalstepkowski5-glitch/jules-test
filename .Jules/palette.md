## 2026-09-10 - Native Form Submission for Login
**Learning:** Attaching login logic to a simple button `click` event breaks native keyboard behavior (e.g., submitting with the Enter key) and creates an accessibility gap.
**Action:** Always wrap inputs in a `<form>`, use associated `<label>`s, a `<button type="submit">`, and bind the JavaScript handler to the form's `submit` event using `event.preventDefault()`.
