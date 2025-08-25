use crate::{
  app_settings::JwtSetting,
  features::{auth::auth_dto::Claims, users::user_dto::UserDto},
};
use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, Validation, decode, encode};
pub struct JwtUtil<'a> {
  pub jwt_config: &'a JwtSetting,
}

impl<'a> JwtUtil<'a> {
  /// Create a new instance of JwtUtil with the provided JWT settings.
  /// # Arguments
  /// * `jwt_config` - A JwtSetting struct containing the JWT configuration.
  /// # Returns
  /// * `JwtUtil` - A new instance of JwtUtil.
  /// # Example
  /// ```
  /// let jwt_settings = JwtSetting {
  ///   secret_key: "your_secret_key".to_string(),
  ///   expiration_minutes: 60,
  ///   issuer: "your_issuer".to_string(),
  ///   audience: "your_audience".to_string(),
  /// };
  /// let jwt_util = JwtUtil::new(jwt_settings);
  /// ```
  /// # Errors
  /// This function does not return errors.
  /// # Panics
  /// This function does not panic.
  /// # Safety
  /// This function is safe to use.
  /// # Notes
  /// This function is part of the JwtUtil implementation.
  /// # See Also
  /// * `JwtSetting` - The struct used for JWT configuration.
  /// # Author
  /// * ROS Sokcheanith
  /// # Date
  /// * 2025-08-25
  pub fn new(jwt_config: &'a JwtSetting) -> Self {
    Self { jwt_config }
  }

  /// Create a JWT token for the given user.
  /// # Arguments
  /// * `user` - A reference to a UserDto struct representing the user for whom the token is to be created.
  /// # Returns
  /// * `Result<String>` - A Result containing the JWT token as a String if successful, or an error if the token creation fails.
  /// # Example
  /// ```
  /// let user = UserDto {
  ///   id: 1,
  ///   user_name: "testuser".to_string(),
  ///   name: "Test User".to_string(),
  ///   email: "test@gmail.com".to_string(),
  /// };
  /// let token = jwt_util.create_token(&user);
  /// ```
  /// # Errors
  /// This function returns an error if the token creation fails.
  /// # Panics
  /// This function does not panic.
  /// # Safety
  /// This function is safe to use.
  /// # Notes
  /// This function is part of the JwtUtil implementation.
  /// # See Also
  /// * `UserDto` - The struct representing the user.
  /// # Author
  /// * ROS Sokcheanith
  /// # Date
  /// * 2025-08-25
  pub fn create_token(&self, user: &UserDto) -> Result<String> {
    let expiration = Utc::now()
      .checked_add_signed(Duration::minutes(self.jwt_config.expiration_minutes as i64))
      .expect("Invalid expiration time")
      .timestamp();
    let claims = Claims {
      sub: user.id,
      exp: expiration as usize,
      iss: self.jwt_config.issuer.clone(),
      aud: self.jwt_config.audience.clone(),
    };

    let token = encode(
      &Header::new(Algorithm::HS256),
      &claims,
      &EncodingKey::from_secret(self.jwt_config.secret_key.as_ref()),
    )?;
    Ok(token)
  }

  /// Decode and validate a JWT token.
  /// # Arguments
  /// * `token` - A string slice representing the JWT token to be decoded and validated.
  /// # Returns
  /// * `Result<Claims>` - A Result containing the Claims struct if the token is valid, or an error if the token is invalid or decoding fails.
  /// # Example
  /// ```
  /// let token = "your_jwt_token";
  /// let claims = jwt_util.decode_token(token);
  /// ```
  /// # Errors
  /// This function returns an error if the token is invalid or decoding fails.
  /// # Panics
  /// This function does not panic.
  /// # Safety
  /// This function is safe to use.
  /// # Notes
  /// This function is part of the JwtUtil implementation.
  /// # See Also
  /// * `Claims` - The struct representing the claims in the JWT token.
  /// # Author
  /// * ROS Sokcheanith
  /// # Date
  /// * 2025-08-25
  pub fn decode_token(&self, token: &str) -> Result<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[self.jwt_config.audience.as_str()]);
    validation.set_issuer(&[self.jwt_config.issuer.clone()]);
    let key = &jsonwebtoken::DecodingKey::from_secret(self.jwt_config.secret_key.as_ref());
    let token_data = decode::<Claims>(token, key, &validation);
    match token_data {
      Ok(data) => Ok(data.claims),
      Err(e) => Err(anyhow::anyhow!("Token decode error: {}", e)),
    }
  }
}
