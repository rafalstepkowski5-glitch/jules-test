## 2024-05-24 - Form Wrapping for Accessibility
**Learning:** Frontend forms and interactive inputs must be wrapped in `<form>` elements and use `<button type="submit">`. JavaScript must listen for the form's `submit` event (using `event.preventDefault()`) rather than simple click listeners to preserve native keyboard submit behaviors (like pressing Enter) and improve screen reader accessibility.
**Action:** Always wrap inputs in forms and hook onto `submit` events for authentication and data entry pages.
