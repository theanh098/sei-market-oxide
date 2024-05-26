use super::api::collection::{SortBy, __path_get_collections};
use super::deserialization::SortDirection;

use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi, ToResponse,
};

#[derive(OpenApi)]
#[openapi(
  info(
    version = "1.1.0",
    license (name = "MIT", url = ""),
    description = "🦀 Axum Api interface",
    contact (name = "thoanh098", url = "https://github.com/theanh098")
  ),
  paths(
      get_collections
    ),
    components(
      schemas(SortDirection,SortBy),
      responses(Empty)
    ),
    modifiers(&BearerSecurity)
  )]
pub struct ApiDoc;

struct BearerSecurity;

// Just trigger modify for BearerSecurity security
#[derive(ToResponse)]
struct Empty;

impl Modify for BearerSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "BearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
