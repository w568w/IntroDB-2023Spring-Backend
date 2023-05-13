pub mod user_type{
    pub const ADMIN: &str = "admin";
    pub const SUPER_ADMIN: &str = "super_admin";
}

pub mod envs{
    pub const DB_URL: &str = "DB_URL";
    pub const JWT_SECRET: &str = "JWT_SECRET";
    pub const ALLOW_ALL_CORS: &str = "ALLOW_ALL_CORS";

}
pub const SECRET_KEY_LENGTH: usize = 32;
pub const ISSUER: &str = "mid";
pub const ACCESS_TOKEN_EXPIRE_SECONDS: i64 = 1800;
pub const REFRESH_TOKEN_EXPIRE_SECONDS: i64 = 3600 * 24 * 7;