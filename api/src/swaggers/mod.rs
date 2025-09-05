use utoipa::{
  Modify, OpenApi,
  openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

use crate::{
  dto::base_res_dto::{BaseResDto, Status},
  features::{
    auth::{auth_dto::LoginReqDto, auth_handler},
    health_check,
    roles::{
      roles_dto::{
        AssignUserRoleReqDto, CreateRoleReqDto, GetUserRolesReqDto, RoleDto, UpdateRoleReqDto,
        UserRolesResDto,
      },
      roles_handler,
    },
    users::{
      user_dto::{GetUserByIdReqDto, UpdateUserReqDto, UserDto, UserRegisterReqDto},
      user_handler,
    },
  },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::register, auth_handler::login,
        auth_handler::logout, health_check::health_checker_handler,
        roles_handler::assign_user_role, roles_handler::create_role,
        roles_handler::get_roles, roles_handler::get_user_roles,
        roles_handler::update_role, user_handler::get_user_by_id,
        user_handler::get_users, user_handler::update_user
    ),
    components(schemas(
        Status,
        UserRegisterReqDto,
        GetUserByIdReqDto,
        UpdateUserReqDto,
        LoginReqDto,
        CreateRoleReqDto,
        UpdateRoleReqDto,
        GetUserRolesReqDto,
        AssignUserRoleReqDto,
        BaseResDto<Vec<UserRolesResDto>>,
        BaseResDto<Vec<RoleDto>>,
        BaseResDto<Vec<UserDto>>,
        GetUserByIdReqDto,
        UpdateUserReqDto,
    )),
    tags(
        (name = "Rust Crud Api Learning", description = "Rust Crud Api Learning")
    ),
    modifiers(&SecurityAddon)
)]

pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    // Take existing components or create new
    let mut components = openapi
      .components
      .take()
      .unwrap_or_else(|| utoipa::openapi::ComponentsBuilder::new().build());

    // Insert security scheme only if not present
    if !components.security_schemes.contains_key("token") {
      components.security_schemes.insert(
        "token".to_string(),
        SecurityScheme::Http(
          HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .build(),
        ),
      );
    }

    // Restore components with schemas and new security scheme
    openapi.components = Some(components);
  }
}
