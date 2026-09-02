## 2024-05-24 - Interactive Inputs Form Wrapper
**Learning:** Inputs connected to authentication or main actions must be wrapped in a `<form>` element. Native form elements correctly trigger the "Enter" key submit behavior automatically out of the box, saving manual keydown listener boilerplate and being more robust for screen readers.
**Action:** Always wrap logical input groupings that need submission in a `<form>` tag and use `<button type="submit">` rather than relying on click listeners.
