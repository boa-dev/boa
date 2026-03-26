
### Initial Idea Submission

* Full Name: Chaitanya Shivram Parab
* Email: [cparab02@gmail.com](mailto:cparab02@gmail.com)
* [Github](https://github.com/Chaitanya-Parab)
* [LinkedIn](https://www.linkedin.com/in/chaitanya-parab/)
* University name: Sindhudurg Shikshan Prasarak Mandal's College of Engineering, Kankavli
* Degree: Bachelor's of Engineering
* Major: Computer Science
* Year: 3rd year
* Expected graduation date: 2027

Project Title: Public API Audit and Stabilization for Boa 1.0
Relevant issues: (https://github.com/boa-dev/boa/issues/4524)


### Idea description:###

Boa is approaching its 1.0 release, making API stability a critical requirement. Public APIs must be carefully reviewed to ensure they are consistent, well-documented and resilient to future changes without introducing breaking changes.My approach will focus on systematically auditing Boa’s public API surface and improving it to align with long-term stability goals. I plan to begin by identifying all publicly exposed items (pub structs, enums, traits and functions) across the codebase. Once identified, I will evaluate whether each item is intentionally public or should be restricted (e.g., converted to pub(crate) where appropriate).

  A key part of the audit will involve:

    1. Replacing public struct fields with getter methods to prevent breaking changes in future modifications.

    2. Introducing #[non_exhaustive] where appropriate to allow extensibility of enums and structs.

    3. Ensuring consistent API design patterns across modules.

    4. Improving documentation for public-facing APIs to clearly define usage and guarantees.

I will take an incremental approach by working module-by-module, submitting small, focused pull requests to ensure reviewability and continuous progress.If time permits, I would also explore tooling or CI checks to detect unintended public API changes, helping maintain stability in the long term.

This project aligns closely with Boa’s goal of achieving a robust and stable 1.0 release, while also improving developer experience and maintainability.