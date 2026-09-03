## 2024-03-24 - Interactive Inputs Form Wrapping
**Learning:** In this codebase, frontend forms and interactive inputs must be wrapped in `<form>` elements and use `<button type="submit">` rather than simple click listeners. This preserves native keyboard submit behaviors (like pressing Enter) and improves screen reader accessibility.
**Action:** When adding or modifying interactive inputs, ensure they are properly wrapped in a `<form>` tag and use an explicit submit button to capture form submission events rather than just listening for button clicks.
