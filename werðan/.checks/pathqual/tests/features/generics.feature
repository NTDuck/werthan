Feature: Generic parameter tracking

  Scenario: Multi-parameter generic lists pass without warnings
    Given the source code:
      """
      pub struct TwoStageEvaluator<H, D> {
          harness: H,
          distiller: D,
      }

      impl<H, D> TwoStageEvaluator<H, D> {
          pub fn with_components(harness: H, distiller: D) -> Self {
              TwoStageEvaluator { harness, distiller }
          }
      }
      """
    When I run pathqual
    Then I get 0 warning(s)

  Scenario: Generic params survive statement semicolons inside bodies
    Given the source code:
      """
      pub struct Pair<H, D> {
          harness: H,
          distiller: D,
      }

      impl<H: ::core::default::Default, D: ::core::default::Default> ::core::default::Default for Pair<H, D> {
          fn default() -> Self {
              let harness = H::default();
              let distiller = D::default();
              Pair { harness, distiller }
          }
      }
      """
    When I run pathqual
    Then I get 0 warning(s)
