## 2024-03-24 - Form Inputs Must Have Labels and Keyboard Support
**Learning:** Found inputs lacking `<label>` elements and a login form lacking enter key support. This pattern of omitting accessible labels and standard keyboard interaction makes the app less intuitive for screen readers and less smooth for all users.
**Action:** Implemented `<label>` elements for inputs by using the `for` attribute referencing the `id` of the input, and added standard "Enter" key submission to passwords inputs. Next time, always check for missing labels and basic keyboard flow in forms.
