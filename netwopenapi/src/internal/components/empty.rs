use super::ApiComponent;
use actix_web::dev::Payload;
use actix_web::{HttpRequest, HttpResponse};
use utoipa::openapi::{RefOr, Schema};

macro_rules! empty_component_impl {
  ($($ty:ty),+) => {
    $(impl ApiComponent for $ty {
      fn child_schemas() -> Vec<(String, RefOr<Schema>)> {
        vec![]
      }
      fn schema() -> Option<(String, RefOr<Schema>)> {
        None
      }
    })+
  };
}

empty_component_impl!(HttpRequest, HttpResponse, Payload, ());
