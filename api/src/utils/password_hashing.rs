use anyhow::Result;
use argon2::{
  Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
  password_hash::{Error as Argon2Error, SaltString, rand_core::OsRng},
};
pub struct PasswordHashing;

impl PasswordHashing {
  /// Hash a password using Argon2 algorithm
  /// # Arguments
  /// * `password` - The plain text password to hash
  /// # Returns
  /// * `Result<String, Argon2Error>` - The hashed password or an error
  /// # Examples
  /// ```
  /// let hashed = PasswordHashing::hash_password("my_password").unwrap();
  /// assert!(PasswordHashing::verify_password("my_password", &hashed));
  /// assert!(!PasswordHashing::verify_password("wrong_password", &hashed));
  /// ```
  /// # Errors
  /// * Returns `Argon2Error` if hashing fails
  /// # Panics
  /// * Panics if the random number generator fails (highly unlikely)
  /// # Safety
  /// * This function is safe to use in concurrent environments
  /// # Performance
  /// * Argon2 is designed to be computationally intensive to resist brute-force attacks
  /// # Security
  /// * Always use a unique salt for each password
  /// # References
  /// * [Argon2 Documentation](https://docs.rs/argon2/)
  /// * [Password Hashing Best Practices](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
  /// # License
  /// * MIT License
  /// # Author
  /// * ROS Sokcheanith
  /// # Date
  /// * 2025-08-25
  pub fn hash_password(password: &str) -> Result<String, Argon2Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
      .hash_password(password.as_bytes(), &salt)?
      .to_string();
    Ok(password_hash)
  }

  /// Verify a password against a hashed password
  /// # Arguments
  /// * `password` - The plain text password to verify
  /// * `hashed` - The hashed password to verify against
  /// # Returns
  /// * `bool` - True if the password matches the hash, false otherwise
  /// # Examples
  /// ```
  /// let hashed = PasswordHashing::hash_password("my_password").unwrap();
  /// assert!(PasswordHashing::verify_password("my_password", &hashed));
  /// assert!(!PasswordHashing::verify_password("wrong_password", &hashed));
  /// ```
  /// # Errors
  /// * Returns false if the hash is invalid or verification fails
  /// # Panics
  /// * Does not panic
  /// # Safety
  /// * This function is safe to use in concurrent environments
  /// # Performance
  /// * Verification is generally faster than hashing
  /// # Security
  /// * Always use a secure hashing algorithm like Argon2
  /// # References
  /// * [Argon2 Documentation](https://docs.rs/argon2/)
  /// * [Password Hashing Best Practices](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
  /// # License
  /// * MIT License
  /// # Author
  /// * ROS Sokcheanith
  /// # Date
  /// * 2025-08-25
  /// # Notes
  /// * Ensure that the hashed password is in the correct format
  /// # Warnings
  /// * Do not use this function with weak hashing algorithms
  pub fn verify_password(password: &str, hashed: &str) -> bool {
    if let Ok(parsed_hash) = PasswordHash::new(hashed) {
      let argon2 = Argon2::default();
      argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
    } else {
      false
    }
  }
}
