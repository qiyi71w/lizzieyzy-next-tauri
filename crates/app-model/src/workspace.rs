use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSharesDto {
    pub left: f64,
    pub right: f64,
}

impl WorkspaceSharesDto {
    pub fn validate(&self) -> Result<(), String> {
        if self.left.is_finite()
            && self.right.is_finite()
            && self.left >= 0.0
            && self.right >= 0.0
            && self.left + self.right < 1.0
        {
            Ok(())
        } else {
            Err("Workspace shares must be finite, nonnegative and sum to less than one.".into())
        }
    }
}
