## 2024-05-24 - Interactive Inputs in Forms
**Learning:** Interactive inputs like login fields were just standalone `<input>` elements relying on a button's `click` listener. This breaks native HTML accessibility features, including submitting the form via the `Enter` key.
**Action:** Always wrap interactive input groups in a semantic `<form>` element, use `<button type="submit">`, and have Javascript listen for the `submit` event (with `event.preventDefault()`) instead of a `click` listener.
