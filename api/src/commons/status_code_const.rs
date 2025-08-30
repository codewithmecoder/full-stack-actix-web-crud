pub struct StatusCodeConst;

impl StatusCodeConst {
  pub const SUCCESS: &'static str = "SUCCESS";
  pub const ERROR: &'static str = "ERROR";
  pub const SERVER_ERROR: &'static str = "SERVER_ERROR";
  pub const NOT_FOUND: &'static str = "NOT_FOUND";
  pub const UNAUTHORIZED: &'static str = "UNAUTHORIZED";
  pub const UQIQUE_CONSTRAINT: &'static str = "UQIQUE_CONSTRAINT";
  pub const TOKEN_MISSING: &'static str = "TOKEN_MISSING";
  pub const FORBIDDEN: &'static str = "FORBIDDEN";
}
